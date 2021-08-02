// static transaction: MemoryDatastore = MemoryDatastore::create("test.db");

mod tests {
    use std::path::Path;
    use std::fs;

    use indradb::{BulkInsertItem, Datastore, EdgeKey, EdgeQuery, EdgeQueryExt, PipeEdgeQuery, PipeVertexQuery, RangeVertexQuery, RocksdbDatastore, RocksdbTransaction, SpecificEdgeQuery, SpecificVertexQuery, Transaction, Type, Vertex, VertexProperty, VertexPropertyQuery, VertexQuery, VertexQueryExt, EdgeDirection};

    use rtag::{GraphConn};
    use uuid::Uuid;

    fn create_new_db() -> RocksdbDatastore {
        let path_name = Uuid::new_v4().to_string();
        let mut p = String::from("tests/test_rocks.db/");
        p.push_str(&path_name.as_str());
        let path = Path::new(&p);
        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }
        RocksdbDatastore::new(path.to_str().unwrap(), Some(1), false).unwrap()

    }

    fn create_new_db_trans() -> GraphConn {
        let db = create_new_db();
        let trans = db.transaction().unwrap();
        GraphConn {
            trans: trans,
        }
    }

    #[test]
    fn test_create_tag() {
        const tag_type: &str = "dummy_type";
        const tag_name: &str = "dummy_tag_name";
        let gconn = create_new_db_trans();
        let tag_v = gconn.create_tag(&Type::new(tag_type).unwrap(), &String::from(tag_name));

        let range = gconn.trans
            .get_vertices(RangeVertexQuery::new(u32::max_value()))
            .unwrap();
        assert_eq!(tag_v.unwrap(), range[0].id);
        assert_eq!(range.len(), 1);
        let props = gconn.trans.get_all_vertex_properties(SpecificVertexQuery::single(range[0].clone().id)).unwrap();
        assert_eq!(Type::new(tag_type).unwrap(), range[0].t);
        let only_prop = &props[0].props[0];
        assert_eq!("Tagname", only_prop.name);
        assert_eq!(String::from(tag_name), only_prop.value);
    }

    #[test]
    fn test_tag_path_or_http() {
        let gconn = create_new_db_trans();
        const tag_type: &str = "dummy_type";
        const tag_name: &str = "dummy_tag_name";
        const path_or_http: &str = "http://www.dummy.com";
        gconn.tag_path_or_http( Some(&Type::new(tag_type).unwrap()), String::from(tag_name), String::from(path_or_http)).unwrap();

        let range = gconn.trans
            .get_vertices(RangeVertexQuery::new(u32::max_value()))
            .unwrap();
        assert_eq!(range.len(), 2);
    }
    
    #[test]
    fn test_tag_path_or_http_multiple_times() {
        let gconn = create_new_db_trans();
        const tag_type: &str = "dummy_type";
        const tag_name: &str = "dummy_tag_name";
        const path_or_http: &str = "http://www.dummy.com";
        gconn.tag_path_or_http( Some(&Type::new(tag_type).unwrap()), String::from(tag_name), String::from(path_or_http)).unwrap();
        gconn.tag_path_or_http( Some(&Type::new(tag_type).unwrap()), String::from(tag_name), String::from(path_or_http)).unwrap();

        let range = gconn.trans
            .get_vertices(RangeVertexQuery::new(u32::max_value()))
            .unwrap();
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn test_find_vertices_with_property_value() {
        let gconn = create_new_db_trans();
        let _ = gconn.create_vertex_with_property(&Type::new("dummy_type1").unwrap(), &String::from("prop1"), &String::from("prop_val1")).unwrap();
        let uuid2 = gconn.create_vertex_with_property(&Type::new("dummy_type2").unwrap(), &String::from("prop2"), &String::from("prop_val2")).unwrap();
        let v = gconn.find_vertices_with_property_value(&String::from("prop2"), &String::from("prop_val2"));

        assert_eq!(v.len(), 1);
        assert_eq!(v[0], uuid2);
    }

    #[test]
    fn test_find_or_replace_case_create() {
        let gconn = create_new_db_trans();
        
        let t = Type::new("test").unwrap();
        let prop= String::from("tag_name");
        let prop_val = String::from("tag1");
        let v = gconn.find_vertex_or_create(&t, &prop_val, &prop);
        let q = gconn.trans.get_vertices(VertexQuery::Range(RangeVertexQuery::new(u32::max_value()))).unwrap();
        assert_eq!(q.len(), 1);

        let found_v = gconn.find_vertices_with_property_value(&prop, &prop_val);
        assert_eq!(found_v[0], v.unwrap());
    }

    #[test]
    fn test_find_or_replace_case_exists() {
        let gconn = create_new_db_trans();
        
        let t = Type::new("test").unwrap();
        let prop= String::from("tag_name");
        let prop_val = String::from("tag1");
        let v_created = gconn.create_vertex_with_property(&t, &prop, &prop_val);
        let v_found = gconn.find_vertex_or_create(&t, &prop_val, &prop);
        let q = gconn.trans.get_vertices(VertexQuery::Range(RangeVertexQuery::new(u32::max_value()))).unwrap();
        assert_eq!(q.len(), 1);

        assert_eq!(v_created.unwrap(), v_found.unwrap());
    }

    #[test]
    fn test_find_edges_between_vertices() {
        let gconn = create_new_db_trans();
        // doesn't exist
        let t = Type::new("test").unwrap();
        let prop= String::from("tag_name");
        let prop_val = String::from("tag1");
        let v1 = gconn.create_vertex_with_property(&t, &prop, &prop_val).unwrap();

        let prop_val = String::from("tag2");
        let v2 = gconn.create_vertex_with_property(&t, &prop, &prop_val).unwrap();

        let n = gconn.find_edges_between_vertices(v1, v2);
        assert_eq!(n, 0);

        // create new edge
        let edge_t = Type::new("t").unwrap();
        let key = EdgeKey::new(v2, edge_t.clone(), v1);
        gconn.trans.create_edge(&key);

        // check again if we find an edge
        let n = gconn.find_edges_between_vertices(v1, v2);
        assert_eq!(n, 1);
    }

    #[test]
    fn test_tag_path_or_http_multiple_calls() {
        const DUMMY_TYPE: &str = "dummy_type";
        let gconn = create_new_db_trans();
        let t_type = Type::new(DUMMY_TYPE).unwrap();
        let t1 = gconn.create_tag( &t_type, &String::from("tag1")).unwrap();
        let t2 = gconn.create_tag( &t_type, &String::from("tag2")).unwrap();
        let t3 = gconn.create_tag( &t_type, &String::from("tag3")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag1"), String::from("tests/fixtures/paper1")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag1"), String::from("tests/fixtures/paper2")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag2"), String::from("tests/fixtures/paper2")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag2"), String::from("tests/fixtures/paper3")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag3"), String::from("tests/fixtures/paper3")).unwrap();
        // run this a second a time
        gconn.tag_path_or_http(Some(&t_type), String::from("tag3"), String::from("tests/fixtures/paper3")).unwrap();

        assert_eq!(gconn.trans.get_vertices(RangeVertexQuery::new(u32::max_value())).unwrap().len(), 6);

        assert_eq!(gconn.trans.get_edge_count(t1, None, EdgeDirection::Outbound).unwrap(), 2);
        assert_eq!(gconn.trans.get_edge_count(t2, None, EdgeDirection::Outbound).unwrap(), 2);
        assert_eq!(gconn.trans.get_edge_count(t3, None, EdgeDirection::Outbound).unwrap(), 1);

        let v = gconn.find_vertices_with_property_value(&String::from("Path"), &String::from("tests/fixtures/paper1"));
        assert_eq!(v.len(), 1);

    }

    #[test]
    #[should_panic(expected = "Found paper1 which is neither a web address nor a path")]
    fn test_tag_path_or_http_panics() {
        const DUMMY_TYPE: &str = "dummy_type";
        let gconn = create_new_db_trans();
        let t_type = Type::new(DUMMY_TYPE).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag1"), String::from("paper1")).unwrap();
    }

    #[test]
    fn test_delete_vertex_with_property() {
        let gconn = create_new_db_trans();
        let t = Type::new("dummy_type1").unwrap();
        let p = String::from("prop1");
        let p_val = String::from("prop_val1");
        gconn.create_vertex_with_property(&t, &p, &p_val).unwrap();
        assert_eq!(gconn.trans.get_vertices(RangeVertexQuery::new(u32::max_value())).unwrap().len(), 1);
        let _ = gconn.delete_vertices_with_property_value(&p, &p_val);
        assert_eq!(gconn.trans.get_vertices(RangeVertexQuery::new(u32::max_value())).unwrap().len(), 0);
    }
    
    #[test]
    fn test_find_tag() {
        let gconn = create_new_db_trans();
        const tag_name: &str = "dummy_tag_name";
        const tag_name2: &str = "other_dummy_tag_name";
        const tag_type: &str = "dummy_type";
        let gconn = create_new_db_trans();
        let tag_v = gconn.create_tag(&Type::new(tag_type).unwrap(), &String::from(tag_name));
        let tag_v = gconn.create_tag(&Type::new(tag_type).unwrap(), &String::from(tag_name2));

        assert_eq!(gconn.find_tag(&String::from(tag_name)).unwrap().len(), 1)
    }

    #[test]
    fn test_show_tags() {
        const DUMMY_TYPE: &str = "dummy_type";
        let gconn = create_new_db_trans();
        let t_type = Type::new(DUMMY_TYPE).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag1"), String::from("tests/fixtures/paper1")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag2"), String::from("tests/fixtures/paper2")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag3"), String::from("tests/fixtures/paper3")).unwrap();

        let mut tag_v = Vec::new();
        tag_v.push(String::from("tag1"));
        tag_v.push(String::from("tag2"));

        gconn.show_tags(tag_v);
    }

    #[test]
    fn test_show_tags_and_associated_items() {
        const DUMMY_TYPE: &str = "dummy_type";
        let gconn = create_new_db_trans();
        let t_type = Type::new(DUMMY_TYPE).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag1"), String::from("tests/fixtures/paper1")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag2"), String::from("tests/fixtures/paper2")).unwrap();
        gconn.tag_path_or_http( Some(&t_type), String::from("tag3"), String::from("tests/fixtures/paper3")).unwrap();

        let mut tag_v = [String::from("tag1"), String::from("tag2")].to_vec();
        let mut items = [String::from("item1"), String::from("item2")].to_vec();
        //tag_v.push(String::from("tag1"));
        //tag_v.push(String::from("tag2"));

        gconn.show_tags_and_associated_items(tag_v, items);
    }
}



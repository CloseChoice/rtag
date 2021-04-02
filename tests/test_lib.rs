// static transaction: MemoryDatastore = MemoryDatastore::create("test.db");

mod tests {
    use std::path::Path;
    use std::fs;

    use indradb::{BulkInsertItem, Datastore, EdgeKey, EdgeQuery, EdgeQueryExt, PipeEdgeQuery, PipeVertexQuery, RangeVertexQuery, RocksdbDatastore, RocksdbTransaction, SpecificEdgeQuery, SpecificVertexQuery, Transaction, Type, Vertex, VertexProperty, VertexPropertyQuery, VertexQuery, VertexQueryExt, EdgeDirection};

    use rtag::{create_and_insert_vertex, create_tag, create_vertex_with_property, find_vertices_with_property_value, tag_path_or_http, find_vertex_or_create, find_edges_between_vertices};
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

    fn create_new_db_trans() -> (RocksdbDatastore, RocksdbTransaction) {
        let db = create_new_db();
        let trans = db.transaction().unwrap();
        (db, trans)
    }

    #[test]
    fn test_something() {
        let (db, trans) = create_new_db_trans();
        create_and_insert_vertex(&db);

        let range = trans
            .get_vertices(RangeVertexQuery::new(u32::max_value()))
            .unwrap();
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn test_create_tag() {
        const tag_type: &str = "dummy_type";
        const tag_name: &str = "dummy_tag_name";
        let (_, trans) = create_new_db_trans();
        let tag_v = create_tag(&trans, &Type::new(tag_type).unwrap(), &String::from(tag_name));

        let range = trans
            .get_vertices(RangeVertexQuery::new(u32::max_value()))
            .unwrap();
        assert_eq!(tag_v.unwrap(), range[0].id);
        assert_eq!(range.len(), 1);
        let props = trans.get_all_vertex_properties(SpecificVertexQuery::single(range[0].clone().id)).unwrap();
        assert_eq!(Type::new(tag_type).unwrap(), range[0].t);
        let only_prop = &props[0].props[0];
        assert_eq!("Tagname", only_prop.name);
        assert_eq!(String::from(tag_name), only_prop.value);
    }

    #[test]
    fn test_tag_path_or_http() {
        let (_, trans) = create_new_db_trans();
        const tag_type: &str = "dummy_type";
        const tag_name: &str = "dummy_tag_name";
        const path_or_http: &str = "http://www.dummy.com";
        tag_path_or_http(&trans,  Some(&Type::new(tag_type).unwrap()), String::from(tag_name), String::from(path_or_http)).unwrap();

        let range = trans
            .get_vertices(RangeVertexQuery::new(u32::max_value()))
            .unwrap();
        assert_eq!(range.len(), 2);
    }
    
    #[test]
    fn test_tag_path_or_http_multiple_times() {
        let (_, trans) = create_new_db_trans();
        const tag_type: &str = "dummy_type";
        const tag_name: &str = "dummy_tag_name";
        const path_or_http: &str = "http://www.dummy.com";
        tag_path_or_http(&trans,  Some(&Type::new(tag_type).unwrap()), String::from(tag_name), String::from(path_or_http)).unwrap();
        tag_path_or_http(&trans,  Some(&Type::new(tag_type).unwrap()), String::from(tag_name), String::from(path_or_http)).unwrap();

        let range = trans
            .get_vertices(RangeVertexQuery::new(u32::max_value()))
            .unwrap();
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn test_find_vertices_with_property_value() {
        let (_, trans) = create_new_db_trans();
        let _ = create_vertex_with_property(&trans, &Type::new("dummy_type1").unwrap(), &String::from("prop1"), &String::from("prop_val1")).unwrap();
        let uuid2 = create_vertex_with_property(&trans, &Type::new("dummy_type2").unwrap(), &String::from("prop2"), &String::from("prop_val2")).unwrap();
        let v = find_vertices_with_property_value(&trans, &String::from("prop2"), &String::from("prop_val2"));

        assert_eq!(v.len(), 1);
        assert_eq!(v[0], uuid2);
    }

    #[test]
    fn test_find_or_replace_case_create() {
        let (_, trans) = create_new_db_trans();
        
        let t = Type::new("test").unwrap();
        let prop= String::from("tag_name");
        let prop_val = String::from("tag1");
        let v = find_vertex_or_create(&trans, &t, &prop_val, &prop);
        let q = trans.get_vertices(VertexQuery::Range(RangeVertexQuery::new(u32::max_value()))).unwrap();
        assert_eq!(q.len(), 1);

        let found_v = find_vertices_with_property_value(&trans, &prop, &prop_val);
        assert_eq!(found_v[0], v.unwrap());
    }

    #[test]
    fn test_find_or_replace_case_exists() {
        let (_, trans) = create_new_db_trans();
        
        let t = Type::new("test").unwrap();
        let prop= String::from("tag_name");
        let prop_val = String::from("tag1");
        let v_created = create_vertex_with_property(&trans, &t, &prop, &prop_val);
        let v_found = find_vertex_or_create(&trans, &t, &prop_val, &prop);
        let q = trans.get_vertices(VertexQuery::Range(RangeVertexQuery::new(u32::max_value()))).unwrap();
        assert_eq!(q.len(), 1);

        assert_eq!(v_created.unwrap(), v_found.unwrap());
    }

    #[test]
    fn test_find_edges_between_vertices() {
        let (_, trans) = create_new_db_trans();
        // doesn't exist
        let t = Type::new("test").unwrap();
        let prop= String::from("tag_name");
        let prop_val = String::from("tag1");
        let v1 = create_vertex_with_property(&trans, &t, &prop, &prop_val).unwrap();

        let prop_val = String::from("tag2");
        let v2 = create_vertex_with_property(&trans, &t, &prop, &prop_val).unwrap();

        let n = find_edges_between_vertices(&trans, v1, v2);
        assert_eq!(n, 0);

        // create new edge
        let edge_t = Type::new("t").unwrap();
        let key = EdgeKey::new(v2, edge_t.clone(), v1);
        trans.create_edge(&key);

        // check again if we find an edge
        let n = find_edges_between_vertices(&trans, v1, v2);
        assert_eq!(n, 1);
    }

    #[test]
    fn test_tag_path_or_http_t() {
        const dummy_type: &str = "dummy_type";
        let (_, trans) = create_new_db_trans();
        let t_type = Type::new(dummy_type).unwrap();
        let t1 = create_tag(&trans, &t_type, &String::from("tag1")).unwrap();
        let t2 = create_tag(&trans, &Type::new(dummy_type).unwrap(), &String::from("tag2")).unwrap();
        let t3 = create_tag(&trans, &Type::new(dummy_type).unwrap(), &String::from("tag3")).unwrap();
        tag_path_or_http(&trans, Some(&Type::new(dummy_type).unwrap()), String::from("tag1"), String::from("paper1")).unwrap();
        tag_path_or_http(&trans, Some(&Type::new(dummy_type).unwrap()), String::from("tag1"), String::from("paper2")).unwrap();
        tag_path_or_http(&trans, Some(&Type::new(dummy_type).unwrap()), String::from("tag2"), String::from("paper2")).unwrap();
        tag_path_or_http(&trans, Some(&Type::new(dummy_type).unwrap()), String::from("tag2"), String::from("paper3")).unwrap();
        tag_path_or_http(&trans, Some(&Type::new(dummy_type).unwrap()), String::from("tag3"), String::from("paper3")).unwrap();
        // run this a second a time
        tag_path_or_http(&trans, Some(&Type::new(dummy_type).unwrap()), String::from("tag3"), String::from("paper3")).unwrap();

        let n = trans.get_vertices(RangeVertexQuery::new(u32::max_value())).unwrap().iter().map(|v| v.id).collect::<Vec<Uuid>>();
        assert_eq!(trans.get_vertices(RangeVertexQuery::new(u32::max_value())).unwrap().len(), 6);

        assert_eq!(trans.get_edge_count(t1, None, EdgeDirection::Outbound).unwrap(), 2);
        assert_eq!(trans.get_edge_count(t2, None, EdgeDirection::Outbound).unwrap(), 2);
        assert_eq!(trans.get_edge_count(t3, None, EdgeDirection::Outbound).unwrap(), 1);

        let v = find_vertices_with_property_value(&trans, &String::from("Path"), &String::from("paper1"));
        println!("VERTEX PROP {:?}", trans.get_all_vertex_properties(RangeVertexQuery::new(u32::max_value())));
        println!("THIS IS THE VERTEX {:?}", v);
        println!("vertices {:?}", trans.get_all_vertex_properties(SpecificVertexQuery::single(v[0]).inbound(u32::max_value()).outbound(u32::max_value()).outbound(u32::max_value()).inbound(u32::max_value())));



    }
}



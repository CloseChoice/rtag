mod struct_tests {
    use std::path::Path;
    use std::fs;

    use indradb::{BulkInsertItem, Datastore, EdgeKey, EdgeQuery, EdgeQueryExt, PipeEdgeQuery, PipeVertexQuery, RangeVertexQuery, RocksdbDatastore, RocksdbTransaction, SpecificEdgeQuery, SpecificVertexQuery, Transaction, Type, Vertex, VertexProperty, VertexPropertyQuery, VertexQuery, VertexQueryExt, EdgeDirection};

    use rtag::{GraphConn, Vtype};
    use rtag::structs::{Tag, Item};
    use uuid::Uuid;


    // todo: use the function from test_lib
    pub fn create_new_db() -> RocksdbDatastore {
        let path_name = Uuid::new_v4().to_string();
        let mut p = String::from("tests/test_rocks.db/");
        p.push_str(&path_name.as_str());
        let path = Path::new(&p);
        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }
        RocksdbDatastore::new(path.to_str().unwrap(), Some(1)).unwrap()

    }

    // todo: use the function from test_lib
    pub fn create_new_db_trans() -> GraphConn {
        let db = create_new_db();
        let trans = db.transaction().unwrap();
        GraphConn {
            trans: trans,
        }
    }

    #[test]
    fn test_create_tag_struct() {
        const tag_type: &str = "dummy_type";
        const tag_name: &str = "dummy_tag_name";
        let gconn = create_new_db_trans();
        let tag_v = gconn.create_tag(&Type::new(tag_type).unwrap(), &String::from(tag_name));

        let t = Tag::new(tag_v.unwrap(), &gconn);
        // todo: add assert here
    }

    #[test]
    fn test_create_tag_struct_with_files() {
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
        // create tag and associated items

        let t = Tag::new(t1, &gconn);
    }

}


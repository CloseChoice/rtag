use uuid::{self, Uuid};
use crate::GraphConn;
use indradb::{
    BulkInsertItem, Datastore, EdgeKey, EdgeQueryExt, RangeVertexQuery, RocksdbDatastore,
    RocksdbTransaction, SpecificVertexQuery, Transaction, Type, Vertex, VertexPropertyQuery, VertexQuery, VertexQueryExt, Edge, Error, EdgeDirection
};

#[derive(Debug)]
pub struct Tag {
    id: Uuid,
    items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    id: Uuid,
    tags: Vec<Tag>,
}

impl Tag {
    pub fn new(id: Uuid, engine: &GraphConn) -> Self {
        let q = SpecificVertexQuery::single(id).inbound().outbound();
        let props = engine.trans.get_all_vertex_properties(q.clone()).unwrap();
        println!("These are props 1 {:?}", props);
        let q = SpecificVertexQuery::single(id).inbound().inbound();
        let props = engine.trans.get_all_vertex_properties(q).unwrap();
        println!("These are props 2 {:?}", props);

        let q = SpecificVertexQuery::single(id).outbound().inbound();
        let props = engine.trans.get_all_vertex_properties(q).unwrap();
        let prop_values: Vec<String> = props.iter().map(|p| p.props[0].value).collect();
        println!("These are props 3 {:?}", props);
        println!("These are props 3 {:?}", prop_values);

        let q = SpecificVertexQuery::single(id).outbound().outbound();
        let props = engine.trans.get_all_vertex_properties(q).unwrap();
        println!("These are props 4 {:?}", props);
        Tag {
            id: id,
            items: Vec::new(),
        }
        // let inbound_files = engine.
    }
}
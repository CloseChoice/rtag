use uuid::{self, Uuid};
use crate::GraphConn;
use indradb::{
    BulkInsertItem, Datastore, EdgeKey, EdgeQueryExt, RangeVertexQuery, RocksdbDatastore,
    RocksdbTransaction, SpecificVertexQuery, Transaction, Type, Vertex, VertexPropertyQuery, VertexQuery, VertexQueryExt, Edge, Error, EdgeDirection,
    NamedProperty, VertexProperties
};

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: Uuid,
    pub itemtype: String,
    pub name: String,
    pub tags: Vec<Tag>,
}

impl From<VertexProperties> for Item {
    fn from(vertex: VertexProperties) -> Self {
        let vertex = vertex.clone();
        Item {
          id: vertex.vertex.id,
          itemtype: vertex.props[0].name.clone(),
          name: vertex.props[0].value.to_string().clone(),
          tags: Vec::new(),
        }
    }
}


impl From<&VertexProperties> for Item {
    fn from(vertex: &VertexProperties) -> Self {
        let vertex = vertex.clone();
        Item {
          id: vertex.vertex.id,
          itemtype: vertex.props[0].name.clone(),
          name: vertex.props[0].value.to_string().clone(),
          tags: Vec::new(),
        }
    }
}

impl From<&VertexProperties> for Tag {
    fn from(vertex: &VertexProperties) -> Self {

        let vertex = vertex.clone();
        Tag {
          id: vertex.vertex.id,
          name: vertex.props[0].value.to_string().clone(),
          items: Vec::new(),
        }
    }
}

impl Tag {
    pub fn new(id: Uuid, engine: &GraphConn) -> Self {
        // first get the name for a given id
        let q = SpecificVertexQuery::single(id);
        let props = engine.trans.get_all_vertex_properties(q).unwrap();
        let name = &props[0].props[0].value;
        println!("This is name: {}", name);
        println!("This is name: {}", name.to_string());

        // get all the neighboring items
        let q = SpecificVertexQuery::single(id).outbound().inbound();
        let props = engine.trans.get_all_vertex_properties(q).unwrap();
        let item_vec: Vec<Item> = props.iter().map(|p| p.into()).collect();
        Tag {
            id: id,
            name: name.to_string(),
            items: item_vec,
        }
    }
}

impl Item {
    pub fn new(id: Uuid, engine: &GraphConn) -> Self {
        // first get the name for a given id
        let q = SpecificVertexQuery::single(id);
        let props = engine.trans.get_all_vertex_properties(q).unwrap();
        let name = &props[0].props[0].name;
        let itemtype = props[0].props[0].name.clone();

        // get all the neighboring items
        let q = SpecificVertexQuery::single(id).outbound().inbound();
        let props = engine.trans.get_all_vertex_properties(q).unwrap();
        let item_vec: Vec<Tag> = props.iter().map(|p| p.into()).collect();
        Item {
            id: id,
            itemtype: itemtype,
            name: name.to_string(),
            tags: item_vec,
        }
    }
}
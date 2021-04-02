
use indradb::{
    BulkInsertItem, Datastore, EdgeKey, EdgeQueryExt, RangeVertexQuery, RocksdbDatastore,
    RocksdbTransaction, SpecificVertexQuery, Transaction, Type, Vertex, VertexPropertyQuery, VertexQuery, VertexQueryExt, Edge
};
use core::fmt;
use std::io;
use uuid::{self, Uuid};
use serde_json::Value as JsonValue;

type Result<T> = ::std::result::Result<T, io::Error>;
#[derive(Debug)]
enum Vtype {
    Edgetype,
    Tagname,
    Path,
    Http,
}

impl fmt::Display for Vtype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

struct GraphConn {
    db: RocksdbDatastore,
    trans: RocksdbTransaction,
}

pub fn create_and_insert_vertex(db: &RocksdbDatastore) -> Vec<uuid::Uuid> {
    let vertex_t = Type::new("test_vertex_type").unwrap();
    let outbound_v = Vertex::new(vertex_t.clone());
    let inbound_v = Vertex::new(vertex_t);

    let items = vec![
        BulkInsertItem::Vertex(outbound_v.clone()),
        BulkInsertItem::Vertex(inbound_v.clone()),
    ];

    db.bulk_insert(items.into_iter()).unwrap();
    vec![outbound_v.id, inbound_v.id]
}

    // create a new vertex of type "tag_type" with the property "tag_name" which gets the value 
    // <name>
pub fn create_tag(trans: &RocksdbTransaction, tag_type: &Type, name: &String) -> Result<uuid::Uuid> {
    create_vertex_with_property(trans, tag_type, &String::from(Vtype::Tagname.to_string()), name)
}

pub fn create_vertex_with_property(trans: &RocksdbTransaction, v_type: &Type, property: &String, property_value: &String) -> Result<uuid::Uuid> {

    let v_id = trans.create_vertex_from_type(v_type.clone()).unwrap();
    let q = SpecificVertexQuery::single(v_id).property(property);
    trans
        .set_vertex_properties(q.clone(), &JsonValue::String(property_value.clone()))
        .unwrap();
    Ok(v_id)
}

pub fn find_vertices_with_property_value(trans: &RocksdbTransaction, property: &String, property_value: &String) -> Vec<uuid::Uuid> {
    let query_v = VertexQuery::Range(RangeVertexQuery::new(u32::max_value()));
    let _prop = property.clone();
    let vertices = trans.get_vertex_properties(VertexPropertyQuery::new(query_v, property)).unwrap();
    // println!("these are the vertices with given property {} and property_value {}: {:?}", prop, property_value, vertices);
    vertices.into_iter().filter(|p| p.value == property_value.clone()).map(|v| v.id).collect::<Vec<uuid::Uuid>>()
}

pub fn tag_path_or_http(trans: &RocksdbTransaction, tag_type: Option<&Type>, tag: String, path_or_http: String) -> Result<()> {
    // todos:
    // - make idempotent, when run twice this will result in two vertices and edges, should only be one
    // - write generic function to get a vertex with a given property -> do we have that already?
    // we need to find a tag with the correct keyword first, then add a property to it.
    let v_type = Type::new(Vtype::Path.to_string()).unwrap();
    let path_v = find_vertex_or_create(trans, &v_type,  &path_or_http, &String::from(Vtype::Path.to_string())).unwrap();
    let tag_v = find_vertex_or_create(trans, tag_type.unwrap(), &tag, &String::from(Vtype::Tagname.to_string())).unwrap();
    let n = find_edges_between_vertices(trans, path_v, tag_v);
    if n == 0 {
        let edge_t = Type::new(Vtype::Edgetype.to_string()).unwrap();
        let key = EdgeKey::new(tag_v, edge_t.clone(), path_v);
        trans.create_edge(&key).unwrap();
        println!("created edge!");
    }
    else {
        println!("edge is already there");
    }
    Ok(())
}


pub fn find_vertex_or_create(trans: &RocksdbTransaction, v_type: &Type, property_value: &String, property: &String) -> Result<Uuid> {
    let v = find_vertices_with_property_value(trans, property, property_value);
    if v.len() == 0 {
        return create_vertex_with_property(trans, v_type, property, property_value)
    }
    Ok(v[0])
}

pub fn find_edges_between_vertices(trans: &RocksdbTransaction, inbound_v: Uuid, outbound_v: Uuid) -> usize {
    let inbound_v_edges = trans.get_edges(SpecificVertexQuery::single(inbound_v).inbound(u32::max_value()));
    let outbound_v_edges = trans.get_edges(SpecificVertexQuery::single(outbound_v).outbound(u32::max_value()));
    let ive = inbound_v_edges.unwrap_or_default();
    let ove = outbound_v_edges.unwrap_or_default();
    ive.into_iter().filter(|v| ove.contains(v)).count()
}

pub fn get_edges_of_vertices(trans: &RocksdbTransaction, vec: Vec<Uuid>) -> Vec<Edge> {
    trans.get_edges(SpecificVertexQuery::new(vec).inbound(u32::max_value())).unwrap()
}
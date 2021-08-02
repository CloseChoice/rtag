
use indradb::{
    BulkInsertItem, Datastore, EdgeKey, EdgeQueryExt, RangeVertexQuery, RocksdbDatastore,
    RocksdbTransaction, SpecificVertexQuery, Transaction, Type, Vertex, VertexPropertyQuery, VertexQuery, VertexQueryExt, Edge, Error
};
use core::fmt;
use std::io;
use uuid::{self, Uuid};
use serde_json::Value as JsonValue;
use prettytable::{Table, Row, Cell, row, cell};
use std::path::{Path, PathBuf};

use std::fs;

type Result<T> = std::result::Result<T, Error>;
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

pub struct GraphConn {
    pub trans: RocksdbTransaction,
}

pub fn create_new_conn(path_name: &str) -> GraphConn {
    let db = RocksdbDatastore::new(path_name, Some(1), false).unwrap();
    let trans = db.transaction().unwrap();
    GraphConn {
        trans: trans,
    }
}


impl GraphConn {
    /// create a new vertex of type "tag_type" with the property "tag_name" which gets the value 
    /// <name>

    pub fn create_tag(&self, tag_type: &Type, name: &String) -> Result<uuid::Uuid> {
        self.create_vertex_with_property(tag_type, &String::from(Vtype::Tagname.to_string()), name)
    }

    pub fn find_tag(&self, name: &String) -> Result<Vec<uuid::Uuid>> {
        Ok(self.find_vertices_with_property_value(&String::from(Vtype::Tagname.to_string()), name))
    }

    pub fn create_vertex_with_property(&self, v_type: &Type, property: &String, property_value: &String) -> Result<uuid::Uuid> {

        let v_id = self.trans.create_vertex_from_type(v_type.clone()).unwrap();
        let q = SpecificVertexQuery::single(v_id).property(property);
        self.trans
            .set_vertex_properties(q.clone(), &JsonValue::String(property_value.clone()))
            .unwrap();
        Ok(v_id)
    }

    pub fn find_vertices_with_property_value(&self, property: &String, property_value: &String) -> Vec<uuid::Uuid> {
        let query_v = VertexQuery::Range(RangeVertexQuery::new(u32::max_value()));
        let _prop = property.clone();
        let vertices = self.trans.get_vertex_properties(VertexPropertyQuery::new(query_v, property)).unwrap();
        // println!("these are the vertices with given property {} and property_value {}: {:?}", prop, property_value, vertices);
        vertices.into_iter().filter(|p| p.value == property_value.clone()).map(|v| v.id).collect::<Vec<uuid::Uuid>>()
    }

    pub fn tag_path_or_http(&self, tag_type: Option<&Type>, tag: String, path_or_http: String) -> Result<()> {
        let v_type = match fs::canonicalize(&path_or_http) {
            Ok(_) => 
                Vtype::Path.to_string(),
            Err(_) =>
                    if path_or_http.starts_with("www") | path_or_http.starts_with("http") {
                        Vtype::Http.to_string()
                    }
                    else {
                        panic!("Found {} which is neither a web address nor a path", path_or_http);
                    }
                };
        let path_v = self.find_vertex_or_create(&Type::new(&v_type).unwrap(),  &path_or_http, &v_type).unwrap();
        let tag_v = self.find_vertex_or_create(tag_type.unwrap(), &tag, &String::from(Vtype::Tagname.to_string())).unwrap();
        let n = self.find_edges_between_vertices(path_v, tag_v);
        if n == 0 {
            let edge_t = Type::new(Vtype::Edgetype.to_string()).unwrap();
            let key = EdgeKey::new(tag_v, edge_t.clone(), path_v);
            self.trans.create_edge(&key).unwrap();
            println!("created edge!");
        }
        else {
            println!("edge is already there");
        }
        Ok(())
    }


    pub fn find_vertex_or_create(&self, v_type: &Type, property_value: &String, property: &String) -> Result<Uuid> {
        let v = self.find_vertices_with_property_value(property, property_value);
        if v.len() == 0 {
            return self.create_vertex_with_property(v_type, property, property_value)
        }
        Ok(v[0])
    }

    pub fn find_edges_between_vertices(&self, inbound_v: Uuid, outbound_v: Uuid) -> usize {
        let inbound_v_edges = self.trans.get_edges(SpecificVertexQuery::single(inbound_v).inbound(u32::max_value()));
        let outbound_v_edges = self.trans.get_edges(SpecificVertexQuery::single(outbound_v).outbound(u32::max_value()));
        let ive = inbound_v_edges.unwrap_or_default();
        let ove = outbound_v_edges.unwrap_or_default();
        ive.into_iter().filter(|v| ove.contains(v)).count()
    }

    pub fn get_edges_of_vertices(&self, vec: Vec<Uuid>) -> Vec<Edge> {
        self.trans.get_edges(SpecificVertexQuery::new(vec).inbound(u32::max_value())).unwrap()
    }

    pub fn delete_vertices_with_property_value(&self, property: &String, property_value: &String) -> Result<()> {
        let v = self.find_vertices_with_property_value(property, property_value);
        self.trans.delete_vertices(SpecificVertexQuery::new(v))?;
        Ok(())
    }

    pub fn show_tags(&self, tags: Vec<String>) {
        let mut table = Table::new();
        table.add_row(Row::from(&["TAG"]));
        for tag in tags.iter() {
            let t = self.find_tag(tag).unwrap();

            if t.len() == 1 {
                let props = self.trans.get_all_vertex_properties(SpecificVertexQuery::new(t)).unwrap();
                for prop in props.iter() {
                    let val = &prop.props[0];
                    table.add_row(row![val.value]);
                }
            }
        }
        table.printstd();
    }

    /// Find 
    fn find_by_hops(&self, num_hops: usize, vtype: Vtype, property_val: String) {
        let v = self.find_vertices_with_property_value(&String::from(vtype.to_string()), &property_val);
        if v.len() == 1 {
            let mut q = SpecificVertexQuery::single(v[0]);
            let s = q.inbound(u32::max_value()).outbound(u32::max_value());
            for _ in 0..num_hops - 1 {
                let s = s.clone().inbound(u32::max_value()).outbound(u32::max_value());
            let props = self.trans.get_all_vertex_properties(s).unwrap();
            }
        }
        match v.len() {
            0 => println!("You supplied the property value {} which can't be found in the database", property_val),
            1 => println!("this is random shit"),
            _ => println!("This is something different"),
        }
    }
    //todo: write this function
    pub fn show_tags_and_associated_items(&self, tags: Vec<String>, items: Vec<String>) {
        let mut table = Table::new();
        table.add_row(Row::from(&["TAG", "ITEM", "ITEM_TYPE"]));
        for tag in tags.iter() {
            let t = self.find_tag(tag).unwrap();

            if t.len() == 1 {
                let props = self.trans.get_all_vertex_properties(SpecificVertexQuery::new(t)).unwrap();
                println!("THESE ARE PROPS {:?}", props);
                for prop in props.iter() {
                    let val = &prop.props[0];
                    table.add_row(row![val.value]);
                }
            }
        }
        table.printstd();
    }
}
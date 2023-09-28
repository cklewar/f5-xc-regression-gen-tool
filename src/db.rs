use indradb::{AllVertexQuery, BulkInsertItem, Edge, Identifier, Json, QueryExt, Vertex, VertexProperties};
use log::{error, info};
use serde_json::{json, to_value};
use uuid::Uuid;

use crate::{constants::*, EDGE_TYPES, EDGES_COUNT, EdgeTypes, PropertyType, VertexTuple, VertexTypes};

pub struct Db {
    pub db: indradb::Database<indradb::MemoryDatastore>,
}

impl Db {
    pub fn new() -> Self {
        Db { db: indradb::MemoryDatastore::new_db() }
    }
    pub fn create_object(&self, object_type: VertexTypes) -> Vertex {
        info!("Create new object of type <{}>...", object_type.name());
        let o = Vertex::new(Identifier::new(object_type.name()).unwrap());
        self.db.create_vertex(&o).expect("panic while creating project db entry");
        self.add_object_properties(&o, &json!({}), PropertyType::Base);
        self.add_object_properties(&o, &json!({}), PropertyType::Gv);
        self.add_object_properties(&o, &json!({}), PropertyType::Module);
        info!("Create new object of type <{}> -> Done", object_type.name());
        o
    }

    fn create_relationship_identifier(&self, a: &Vertex, b: &Vertex) -> Identifier {
        info!("Create relationship identifier for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let i = Identifier::new(self.get_relationship_type(&a, &b)).unwrap();
        info!("Create relationship identifier for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        i
    }

    pub fn create_relationship(&self, a: &Vertex, b: &Vertex) -> bool {
        info!("Create relationship for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let i = self.create_relationship_identifier(&a, &b);
        let e = Edge::new(a.id, i, b.id);
        let status = self.db.create_edge(&e).expect(&format!("panic build relationship between {} and {}", a.t.as_str(), b.t.as_str()));
        info!("Create relationship for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        status
    }

    pub fn add_object_properties<T: serde::Serialize>(&self, object: &Vertex, value: &T, property_type: PropertyType) {
        info!("Add new property to object <{}>...", object.t.to_string());
        let v = to_value(value).unwrap();
        let p: BulkInsertItem;

        match property_type {
            PropertyType::Gv => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_GV)
                    .unwrap(), Json::new(v.clone()));
            }
            PropertyType::Base => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_BASE)
                    .unwrap(), Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_MODULE)
                    .unwrap(), Json::new(v.clone()));
            }
        }

        self.db.bulk_insert(vec![p]).unwrap();
        info!("Add new property to object <{}> -> Done", object.t.to_string());
    }

    #[allow(dead_code)]
    fn add_relationship_properties<T: serde::Serialize>(&self, object: &Edge, value: &T, property_type: PropertyType) {
        info!("Add new property to relationship <{}>...", object.t.to_string());
        let v = to_value(value).unwrap();
        let p: BulkInsertItem;

        match property_type {
            PropertyType::Gv => {
                p = BulkInsertItem::EdgeProperty(object.clone(), Identifier::new(PROPERTY_TYPE_GV)
                    .unwrap(), Json::new(v.clone()));
            }
            PropertyType::Base => {
                p = BulkInsertItem::EdgeProperty(object.clone(), Identifier::new(PROPERTY_TYPE_BASE)
                    .unwrap(), Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::EdgeProperty(object.clone(), Identifier::new(PROPERTY_TYPE_MODULE)
                    .unwrap(), Json::new(v.clone()));
            }
        }
        self.db.bulk_insert(vec![p]).unwrap();
        info!("Add new property to relationship <{}> -> Done", object.t.to_string());
    }

    #[allow(dead_code)]
    fn get_relationship_count() -> usize {
        info!("Relationship count: <{}>", *EDGES_COUNT);
        *EDGES_COUNT
    }

    fn get_relationship_type(&self, a: &Vertex, b: &Vertex) -> &str {
        info!("Get relationship type for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        error!("RELA -----> {:?}, {:?}", a.t.to_string(), b.t.to_string());
        let e = EDGE_TYPES.get(&VertexTuple(a.t.to_string(), b.t.to_string())).unwrap();
        info!("Get relationship type for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        e
    }

    pub(crate) fn get_all_objects(&self) -> Option<Vec<Vertex>> {
        let q = AllVertexQuery;
        let result = self.db.get(q);
        indradb::util::extract_vertices(result.unwrap())
    }

    pub(crate) fn get_all_edges(&self) -> Option<Vec<Edge>> {
        let q = AllVertexQuery.include().outbound();
        let result = self.db.get(q.unwrap()).unwrap();
        indradb::util::extract_edges(result)
    }

    pub(crate) fn get_object(&self, id: &Uuid) -> Vertex {
        let q = self.db.get(indradb::SpecificVertexQuery::single(*id));
        let _objs = indradb::util::extract_vertices(q.unwrap());
        let objs = _objs.unwrap();
        let o = objs.get(0).unwrap();
        o.clone()
    }

    pub fn get_object_with_properties(&self, id: &Uuid) -> VertexProperties {
        let obj = self.db.get(indradb::SpecificVertexQuery::single(*id).properties().unwrap());
        let a = indradb::util::extract_vertex_properties(obj.unwrap()).unwrap();
        a.get(0).unwrap().clone()
    }

    pub fn get_object_neighbour_out(&self, id: &Uuid, identifier: EdgeTypes) -> Vertex {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.db.get(indradb::SpecificVertexQuery::single(*id).outbound().unwrap().t(i));
        let id = indradb::util::extract_edges(o.unwrap()).unwrap().get(0).unwrap().inbound_id;
        self.get_object(&id)
    }

    pub fn get_object_neighbours_out(&self, id: &Uuid, identifier: EdgeTypes) -> Vec<Vertex> {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.db.get(indradb::SpecificVertexQuery::single(*id).outbound().unwrap().t(i));
        let mut objs: Vec<Vertex> = Vec::new();

        for item in indradb::util::extract_edges(o.unwrap()).unwrap().iter() {
            objs.push(self.get_object(&item.inbound_id));
        }
        objs
    }

    pub fn get_object_neighbour_with_properties_out(&self, id: &Uuid, identifier: EdgeTypes) -> Option<VertexProperties> {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.db.get(indradb::SpecificVertexQuery::single(*id).outbound().unwrap().t(i));

        return match indradb::util::extract_edges(o.unwrap()).unwrap().get(0) {
            Some(v) => {
                let id = v.inbound_id;
                Some(self.get_object_with_properties(&id))
            }
            None => {
                None
            }
        };
    }

    pub fn get_object_neighbours_with_properties_out(&self, id: &Uuid, identifier: EdgeTypes) -> Vec<VertexProperties> {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.db.get(indradb::SpecificVertexQuery::single(*id).outbound().unwrap().t(i));
        let mut objs: Vec<VertexProperties> = Vec::new();

        for item in indradb::util::extract_edges(o.unwrap()).unwrap().iter() {
            objs.push(self.get_object_with_properties(&item.inbound_id));
        }
        objs
    }

    pub fn get_object_neighbours_with_properties_in(&self, id: &Uuid, identifier: EdgeTypes) -> Vec<VertexProperties> {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.db.get(indradb::SpecificVertexQuery::single(*id).inbound().unwrap().t(i));
        let mut objs: Vec<VertexProperties> = Vec::new();

        for item in indradb::util::extract_edges(o.unwrap()).unwrap().iter() {
            objs.push(self.get_object_with_properties(&item.outbound_id));
        }
        objs
    }

    pub fn get_object_properties(&self, object: &Vertex) -> Option<VertexProperties> {
        info!("Get object <{}> properties...", object.t.as_str());
        let b = indradb::SpecificVertexQuery::new(vec!(object.id)).properties().unwrap();
        let _r = self.db.get(b);
        return match _r {
            Ok(qov) => {
                let vp = indradb::util::extract_vertex_properties(qov);
                if vp.clone().unwrap().get(0).is_some() {
                    info!("Get object <{}> properties -> Done.", object.t.as_str());
                    Some(vp.unwrap().get(0).unwrap().clone())
                } else {
                    info!("Get object <{}> properties -> Done.", object.t.as_str());
                    None
                }
            }
            Err(e) => {
                error!("Error in properties query: {}", &e);
                None
            }
        };
    }
}
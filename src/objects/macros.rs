macro_rules! implement_object_ext {
    ($($structure:ident),* ) => {
        $(
            impl ObjectExt for $structure<'_> {
                fn get_id(&self) -> Uuid { self.object.get_id() }
                fn get_object(&self) -> Vertex { self.object.get_object() }
                fn get_id_path(&self) -> &IdPath { self.object.get_id_path() }
                fn get_module_cfg(&self) -> Map<String, Value> { self.object.get_module_cfg() }
                fn get_base_properties(&self) -> Map<String, Value> { self.object.get_base_properties() }
                fn get_module_properties(&self) -> Map<String, Value> { self.object.get_module_properties() }
                fn get_object_with_properties(&self) -> VertexProperties { self.object.get_object_with_properties() }
                fn add_base_properties(&self, value: Value) { self.object.add_base_properties(value) }
                fn add_module_properties(&self, value: Value) { self.object.add_module_properties(value) }
                fn insert_base_property(&self, key: String, value: Value) {
                    self.object.insert_base_property(key, value)
                }
                fn insert_module_property(&self, key: String, value: Value) {
                    self.object.insert_module_property(key, value)
                }
            }
        )*
    };
}

pub(crate) use implement_object_ext;
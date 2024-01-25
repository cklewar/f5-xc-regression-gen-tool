macro_rules! implement_object_ext {
    ($($structure:ident),* ) => {
        $(
            impl ObjectExt for $structure<'_> {
                fn get_id(&self) -> Uuid { self.object.get_id() }
                fn fn_a(&self) -> String { self.object.fn_a() }
                fn fn_b(&self) -> String { self.object.fn_b() }
                fn get_object(&self) -> Vertex { self.object.get_object() }
                fn get_id_path(&self) -> &IdPath { self.object.get_id_path() }
                fn get_module_cfg(&self) -> Map<String, Value> { self.object.get_module_cfg() }
                fn get_base_properties(&self) -> Map<String, Value> { self.object.get_base_properties() }
                fn get_module_properties(&self) -> Map<String, Value> { self.object.get_module_properties() }
                fn insert_module_properties(&self, key: String, value: Value) {
                    self.object.insert_module_properties(key, value)
                }
            }
        )*
    };
}

pub(crate) use implement_object_ext;
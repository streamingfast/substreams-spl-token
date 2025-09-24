// @generated
// @@protoc_insertion_point(attribute:schema)
pub mod schema {
    include!("schema.rs");
    // @@protoc_insertion_point(schema)
}
pub mod sf {
    pub mod solana {
        pub mod r#type {
            // @@protoc_insertion_point(attribute:sf.solana.type.v1)
            pub mod v1 {
                include!("sf.solana.type.v1.rs");
                // @@protoc_insertion_point(sf.solana.type.v1)
            }
        }
        pub mod spl {
            pub mod v1 {
                // @@protoc_insertion_point(attribute:sf.solana.spl.v1.type)
                pub mod r#type {
                    include!("sf.solana.spl.v1.type.rs");
                    // @@protoc_insertion_point(sf.solana.spl.v1.type)
                }
            }
        }
    }
    pub mod substreams {
        pub mod solana {
            pub mod spl {
                // @@protoc_insertion_point(attribute:sf.substreams.solana.spl.v1)
                pub mod v1 {
                    include!("sf.substreams.solana.spl.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.solana.spl.v1)
                }
            }
            // @@protoc_insertion_point(attribute:sf.substreams.solana.v1)
            pub mod v1 {
                include!("sf.substreams.solana.v1.rs");
                // @@protoc_insertion_point(sf.substreams.solana.v1)
            }
        }
    }
}

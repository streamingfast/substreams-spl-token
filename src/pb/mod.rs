// @generated
// @@protoc_insertion_point(attribute:parquet)
pub mod parquet {
    include!("parquet.rs");
    // @@protoc_insertion_point(parquet)
}
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
    // @@protoc_insertion_point(attribute:sf.substreams)
    pub mod substreams {
        include!("sf.substreams.rs");
        // @@protoc_insertion_point(sf.substreams)
        pub mod index {
            // @@protoc_insertion_point(attribute:sf.substreams.index.v1)
            pub mod v1 {
                include!("sf.substreams.index.v1.rs");
                // @@protoc_insertion_point(sf.substreams.index.v1)
            }
        }
        pub mod rpc {
            // @@protoc_insertion_point(attribute:sf.substreams.rpc.v2)
            pub mod v2 {
                include!("sf.substreams.rpc.v2.rs");
                // @@protoc_insertion_point(sf.substreams.rpc.v2)
            }
        }
        pub mod sink {
            pub mod service {
                // @@protoc_insertion_point(attribute:sf.substreams.sink.service.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.service.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.service.v1)
                }
            }
            pub mod sql {
                pub mod service {
                    // @@protoc_insertion_point(attribute:sf.substreams.sink.sql.service.v1)
                    pub mod v1 {
                        include!("sf.substreams.sink.sql.service.v1.rs");
                        // @@protoc_insertion_point(sf.substreams.sink.sql.service.v1)
                    }
                }
                // @@protoc_insertion_point(attribute:sf.substreams.sink.sql.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.sql.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.sql.v1)
                }
            }
        }
        pub mod solana {
            // @@protoc_insertion_point(attribute:sf.substreams.solana.v1)
            pub mod v1 {
                include!("sf.substreams.solana.v1.rs");
                // @@protoc_insertion_point(sf.substreams.solana.v1)
            }
        }
        // @@protoc_insertion_point(attribute:sf.substreams.v1)
        pub mod v1 {
            include!("sf.substreams.v1.rs");
            // @@protoc_insertion_point(sf.substreams.v1)
        }
    }
}
pub mod sol {
    pub mod instructions {
        // @@protoc_insertion_point(attribute:sol.instructions.v1)
        pub mod v1 {
            include!("sol.instructions.v1.rs");
            // @@protoc_insertion_point(sol.instructions.v1)
        }
    }
    pub mod transactions {
        // @@protoc_insertion_point(attribute:sol.transactions.v1)
        pub mod v1 {
            include!("sol.transactions.v1.rs");
            // @@protoc_insertion_point(sol.transactions.v1)
        }
    }
}

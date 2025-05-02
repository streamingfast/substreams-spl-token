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

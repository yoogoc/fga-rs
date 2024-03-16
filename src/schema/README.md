# Schema

This is using a schema similar to [SpiceDB Schema](https://authzed.com/docs/spicedb/concepts/schema).

it can parse the schema and generate the following:

```rust
Schema {
            definitions: vec![
                Definition {
                    name: "user",
                    relations: vec![],
                    permissions: vec![],
                },
                Definition {
                    name: "resource",
                    relations: vec![Relation {
                        name: "manager",
                        subject: vec![
                            Associations::Single("user"),
                            Associations::SubjectSet(SubjectSet {
                                group: "usergroup",
                                role: "member",
                            }),
                            Associations::SubjectSet(SubjectSet {
                                group: "usergroup",
                                role: "manager",
                            }),
                        ],
                    }],
                    permissions: vec![Permission {
                        name: "view",
                        permissions: vec![Permissions::Single("viewer"), Permissions::Single("manager")],
                    }],
                },
            ],
        }
        ```
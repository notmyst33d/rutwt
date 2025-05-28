extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, Data, Fields, FieldsNamed};

fn error(span: proc_macro2::Span, message: &'static str) -> proc_macro::TokenStream {
    syn::Error::new(span, message).into_compile_error().into()
}

#[proc_macro_derive(Migrate, attributes(table, seq_key))]
pub fn migrate_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_migrate(&ast)
}

fn impl_migrate_fields_push(fields: &FieldsNamed) -> proc_macro2::TokenStream {
    let field_name = fields.named.iter().map(|field| &field.ident);
    quote! {
        #(fields_builder.push(stringify!(#field_name));)*
    }
}

fn impl_migrate_fields_bind(fields: &FieldsNamed) -> proc_macro2::TokenStream {
    let field_name = fields.named.iter().map(|field| &field.ident);
    quote! {
        #(values_builder.push_bind(&row.#field_name);)*
    }
}

fn impl_migrate_fields_seq_sync(
    table_name: &TokenStream,
    fields: &FieldsNamed,
) -> proc_macro2::TokenStream {
    if let Some(field) = fields.named.iter().find(|f| {
        f.attrs
            .iter()
            .find(|a| a.path().is_ident("seq_key"))
            .is_some()
    }) {
        let name = &field.ident;
        quote! {
            sqlx::query(concat!("SELECT setval('", #table_name, "_", stringify!(#name), "_seq', max(", stringify!(#name), ")) FROM ", #table_name)).execute(pg).await.expect("failed to sync seq");
        }
    } else {
        quote! {}
    }
}

fn impl_migrate(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;
    let Some(table_attr) = ast.attrs.iter().find(|a| a.path().is_ident("table")) else {
        return error(name.span(), "`table` attribute is required");
    };

    let table_name = &table_attr.meta.require_list().unwrap().tokens;

    if let Data::Struct(ref data) = ast.data {
        if let Fields::Named(ref named) = data.fields {
            let fields_push = impl_migrate_fields_push(named);
            let fields_bind = impl_migrate_fields_bind(named);
            let sync = impl_migrate_fields_seq_sync(table_name, named);
            quote! {
                impl #name {
                    pub async fn migrate(sqlite: &SqlitePool, pg: &PgPool) {
                        let mut stream = sqlx::query_as::<_, Self>(concat!("SELECT * FROM ", #table_name)).fetch(sqlite);
                        while let Some(row) = stream.try_next().await.expect("cannot read record") {
                            let mut builder = QueryBuilder::new(concat!("INSERT INTO ", #table_name, " ("));
                            let mut fields_builder = builder.separated(", ");
                            #fields_push
                            builder.push(") VALUES (");
                            let mut values_builder = builder.separated(", ");
                            #fields_bind
                            builder.push(")");
                            builder.build().execute(pg).await.expect("failed to migrate");
                            #sync
                        }
                    }
                }
            }
            .into()
        } else {
            error(name.span(), "Cannot use `Migrate` on unnamed fields")
        }
    } else {
        error(name.span(), "Cannot use `Migrate` on non-structs")
    }
}

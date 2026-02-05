//! Salesforce schema modeling for SOQL to SQL conversion

use std::collections::HashMap;

/// Complete Salesforce org schema for SQL translation
#[derive(Debug, Clone, Default)]
pub struct SalesforceSchema {
    /// Map from SObject API name (case-insensitive key) to object description
    objects: HashMap<String, SObjectDescribe>,
}

impl SalesforceSchema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an SObject to the schema
    pub fn add_object(&mut self, object: SObjectDescribe) {
        self.objects.insert(object.name.to_lowercase(), object);
    }

    /// Get an SObject by API name (case-insensitive)
    pub fn get_object(&self, name: &str) -> Option<&SObjectDescribe> {
        self.objects.get(&name.to_lowercase())
    }

    /// Get a mutable reference to an SObject
    pub fn get_object_mut(&mut self, name: &str) -> Option<&mut SObjectDescribe> {
        self.objects.get_mut(&name.to_lowercase())
    }

    /// Get all objects
    pub fn objects(&self) -> impl Iterator<Item = &SObjectDescribe> {
        self.objects.values()
    }

    /// Check if an object exists
    pub fn has_object(&self, name: &str) -> bool {
        self.objects.contains_key(&name.to_lowercase())
    }
}

/// Description of a Salesforce SObject
#[derive(Debug, Clone)]
pub struct SObjectDescribe {
    /// API name (e.g., "Account", "Custom_Object__c")
    pub name: String,
    /// SQL table name (typically snake_case: "account", "custom_object__c")
    pub table_name: String,
    /// Label for display
    pub label: String,
    /// Map from field API name (case-insensitive) to field description
    fields: HashMap<String, FieldDescribe>,
    /// Child relationships (for subqueries)
    pub child_relationships: Vec<ChildRelationship>,
    /// Whether this object supports record types
    pub has_record_types: bool,
}

impl SObjectDescribe {
    /// Create a new SObject description
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let table_name = to_snake_case(&name);
        Self {
            label: name.clone(),
            name,
            table_name,
            fields: HashMap::new(),
            child_relationships: Vec::new(),
            has_record_types: false,
        }
    }

    /// Set the SQL table name
    pub fn with_table_name(mut self, table_name: impl Into<String>) -> Self {
        self.table_name = table_name.into();
        self
    }

    /// Set the label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Add a field to the object
    pub fn add_field(&mut self, field: FieldDescribe) {
        self.fields.insert(field.name.to_lowercase(), field);
    }

    /// Get a field by API name (case-insensitive)
    pub fn get_field(&self, name: &str) -> Option<&FieldDescribe> {
        self.fields.get(&name.to_lowercase())
    }

    /// Get all fields
    pub fn fields(&self) -> impl Iterator<Item = &FieldDescribe> {
        self.fields.values()
    }

    /// Check if a field exists
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(&name.to_lowercase())
    }

    /// Add a child relationship
    pub fn add_child_relationship(&mut self, relationship: ChildRelationship) {
        self.child_relationships.push(relationship);
    }

    /// Find a child relationship by name
    pub fn get_child_relationship(&self, name: &str) -> Option<&ChildRelationship> {
        let lower = name.to_lowercase();
        self.child_relationships
            .iter()
            .find(|r| r.relationship_name.to_lowercase() == lower)
    }
}

/// Description of a Salesforce field
#[derive(Debug, Clone)]
pub struct FieldDescribe {
    /// API name (e.g., "AccountId", "Custom_Field__c")
    pub name: String,
    /// SQL column name (typically snake_case)
    pub column_name: String,
    /// Salesforce field type
    pub field_type: SalesforceFieldType,
    /// For lookup/master-detail fields: object(s) this references
    pub reference_to: Option<Vec<String>>,
    /// Relationship name for parent traversal (e.g., "Account" for AccountId)
    pub relationship_name: Option<String>,
    /// Is this a polymorphic field (e.g., OwnerId -> User|Group)
    pub is_polymorphic: bool,
    /// Field length for strings
    pub length: Option<u32>,
    /// Precision for numeric types
    pub precision: Option<u8>,
    /// Scale for numeric types
    pub scale: Option<u8>,
    /// Whether the field can be null
    pub nillable: bool,
    /// For picklists: valid values
    pub picklist_values: Option<Vec<String>>,
}

impl FieldDescribe {
    /// Create a new field description
    pub fn new(name: impl Into<String>, field_type: SalesforceFieldType) -> Self {
        let name = name.into();
        let column_name = to_snake_case(&name);
        Self {
            name,
            column_name,
            field_type,
            reference_to: None,
            relationship_name: None,
            is_polymorphic: false,
            length: None,
            precision: None,
            scale: None,
            nillable: true,
            picklist_values: None,
        }
    }

    /// Set the SQL column name
    pub fn with_column_name(mut self, column_name: impl Into<String>) -> Self {
        self.column_name = column_name.into();
        self
    }

    /// Set this as a lookup field
    pub fn with_reference(mut self, reference_to: impl Into<String>) -> Self {
        self.reference_to = Some(vec![reference_to.into()]);
        self
    }

    /// Set this as a polymorphic lookup field
    pub fn with_polymorphic_reference(mut self, reference_to: Vec<String>) -> Self {
        self.reference_to = Some(reference_to);
        self.is_polymorphic = true;
        self
    }

    /// Set the relationship name
    pub fn with_relationship_name(mut self, name: impl Into<String>) -> Self {
        self.relationship_name = Some(name.into());
        self
    }

    /// Set the field length
    pub fn with_length(mut self, length: u32) -> Self {
        self.length = Some(length);
        self
    }

    /// Set the precision and scale
    pub fn with_precision(mut self, precision: u8, scale: u8) -> Self {
        self.precision = Some(precision);
        self.scale = Some(scale);
        self
    }

    /// Set whether the field is nillable
    pub fn with_nillable(mut self, nillable: bool) -> Self {
        self.nillable = nillable;
        self
    }

    /// Set picklist values
    pub fn with_picklist_values(mut self, values: Vec<String>) -> Self {
        self.picklist_values = Some(values);
        self
    }

    /// Check if this is a relationship field
    pub fn is_relationship(&self) -> bool {
        self.reference_to.is_some()
    }
}

/// Child relationship (for subqueries like SELECT ... FROM Contacts)
#[derive(Debug, Clone)]
pub struct ChildRelationship {
    /// Relationship name used in SOQL (e.g., "Contacts", "Opportunities")
    pub relationship_name: String,
    /// Child object API name (e.g., "Contact")
    pub child_object: String,
    /// Field on child object (e.g., "AccountId")
    pub field: String,
}

impl ChildRelationship {
    pub fn new(
        relationship_name: impl Into<String>,
        child_object: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self {
            relationship_name: relationship_name.into(),
            child_object: child_object.into(),
            field: field.into(),
        }
    }
}

/// Salesforce field types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SalesforceFieldType {
    Id,
    String,
    TextArea,
    LongTextArea,
    RichTextArea,
    Boolean,
    Integer,
    Double,
    Currency,
    Percent,
    Date,
    DateTime,
    Time,
    Phone,
    Email,
    Url,
    Picklist,
    MultiPicklist,
    Lookup,
    MasterDetail,
    Reference, // Polymorphic
    Address,   // Compound
    Location,  // Geolocation compound
    Auto,      // Autonumber
}

impl SalesforceFieldType {
    /// Get the appropriate SQL type for this field type
    pub fn to_sql_type(&self) -> &'static str {
        match self {
            SalesforceFieldType::Id => "TEXT",
            SalesforceFieldType::String => "TEXT",
            SalesforceFieldType::TextArea => "TEXT",
            SalesforceFieldType::LongTextArea => "TEXT",
            SalesforceFieldType::RichTextArea => "TEXT",
            SalesforceFieldType::Boolean => "BOOLEAN",
            SalesforceFieldType::Integer => "INTEGER",
            SalesforceFieldType::Double => "REAL",
            SalesforceFieldType::Currency => "REAL",
            SalesforceFieldType::Percent => "REAL",
            SalesforceFieldType::Date => "DATE",
            SalesforceFieldType::DateTime => "TIMESTAMP",
            SalesforceFieldType::Time => "TIME",
            SalesforceFieldType::Phone => "TEXT",
            SalesforceFieldType::Email => "TEXT",
            SalesforceFieldType::Url => "TEXT",
            SalesforceFieldType::Picklist => "TEXT",
            SalesforceFieldType::MultiPicklist => "TEXT",
            SalesforceFieldType::Lookup => "TEXT",
            SalesforceFieldType::MasterDetail => "TEXT",
            SalesforceFieldType::Reference => "TEXT",
            SalesforceFieldType::Address => "TEXT", // JSON in practice
            SalesforceFieldType::Location => "TEXT", // JSON in practice
            SalesforceFieldType::Auto => "TEXT",
        }
    }
}

/// Convert a Salesforce API name to snake_case for SQL
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut chars = s.chars().peekable();
    let mut prev_was_upper = false;
    let mut prev_was_underscore = true; // Treat start as after underscore

    while let Some(c) = chars.next() {
        if c == '_' {
            result.push('_');
            prev_was_underscore = true;
            prev_was_upper = false;
        } else if c.is_uppercase() {
            // Add underscore before uppercase if:
            // - Not at start
            // - Previous char was lowercase OR next char is lowercase (for sequences like "HTTPApi" -> "http_api")
            if !prev_was_underscore {
                let next_is_lower = chars.peek().map(|c| c.is_lowercase()).unwrap_or(false);
                if !prev_was_upper || next_is_lower {
                    result.push('_');
                }
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_was_upper = true;
            prev_was_underscore = false;
        } else {
            result.push(c.to_lowercase().next().unwrap());
            prev_was_upper = false;
            prev_was_underscore = false;
        }
    }

    result
}

/// Builder for creating standard Salesforce schemas
pub struct SchemaBuilder {
    schema: SalesforceSchema,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self {
            schema: SalesforceSchema::new(),
        }
    }

    /// Add standard system fields to an object
    pub fn add_standard_fields(object: &mut SObjectDescribe) {
        // Id field
        object.add_field(
            FieldDescribe::new("Id", SalesforceFieldType::Id)
                .with_nillable(false),
        );

        // Name field (most objects have this)
        object.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));

        // System audit fields
        object.add_field(
            FieldDescribe::new("CreatedById", SalesforceFieldType::Lookup)
                .with_reference("User")
                .with_relationship_name("CreatedBy"),
        );
        object.add_field(FieldDescribe::new("CreatedDate", SalesforceFieldType::DateTime));
        object.add_field(
            FieldDescribe::new("LastModifiedById", SalesforceFieldType::Lookup)
                .with_reference("User")
                .with_relationship_name("LastModifiedBy"),
        );
        object.add_field(FieldDescribe::new(
            "LastModifiedDate",
            SalesforceFieldType::DateTime,
        ));
        object.add_field(FieldDescribe::new(
            "SystemModstamp",
            SalesforceFieldType::DateTime,
        ));

        // Soft delete
        object.add_field(FieldDescribe::new("IsDeleted", SalesforceFieldType::Boolean));

        // Owner (for objects with sharing)
        object.add_field(
            FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
                .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
                .with_relationship_name("Owner"),
        );
    }

    /// Add a standard object with common fields
    pub fn with_standard_object(mut self, name: &str) -> Self {
        let mut object = SObjectDescribe::new(name);
        Self::add_standard_fields(&mut object);
        self.schema.add_object(object);
        self
    }

    /// Add a custom object
    pub fn with_object(mut self, object: SObjectDescribe) -> Self {
        self.schema.add_object(object);
        self
    }

    /// Build the schema
    pub fn build(self) -> SalesforceSchema {
        self.schema
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_conversion() {
        assert_eq!(to_snake_case("Account"), "account");
        assert_eq!(to_snake_case("AccountId"), "account_id");
        assert_eq!(to_snake_case("Custom_Object__c"), "custom_object__c");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("XMLParser"), "xml_parser");
        assert_eq!(to_snake_case("getHTTPResponse"), "get_http_response");
    }

    #[test]
    fn test_schema_lookup() {
        let mut schema = SalesforceSchema::new();

        let mut account = SObjectDescribe::new("Account");
        account.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id));
        account.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
        schema.add_object(account);

        // Case-insensitive lookup
        assert!(schema.get_object("Account").is_some());
        assert!(schema.get_object("account").is_some());
        assert!(schema.get_object("ACCOUNT").is_some());

        let obj = schema.get_object("account").unwrap();
        assert_eq!(obj.name, "Account");
        assert_eq!(obj.table_name, "account");

        // Field lookup
        assert!(obj.get_field("Id").is_some());
        assert!(obj.get_field("id").is_some());
        assert!(obj.get_field("name").is_some());
    }

    #[test]
    fn test_relationship_field() {
        let field = FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Account");

        assert!(field.is_relationship());
        assert!(!field.is_polymorphic);
        assert_eq!(field.reference_to, Some(vec!["Account".to_string()]));
        assert_eq!(field.relationship_name, Some("Account".to_string()));
    }

    #[test]
    fn test_polymorphic_field() {
        let field = FieldDescribe::new("WhatId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec![
                "Account".to_string(),
                "Opportunity".to_string(),
                "Case".to_string(),
            ])
            .with_relationship_name("What");

        assert!(field.is_relationship());
        assert!(field.is_polymorphic);
        assert_eq!(
            field.reference_to,
            Some(vec![
                "Account".to_string(),
                "Opportunity".to_string(),
                "Case".to_string()
            ])
        );
    }

    #[test]
    fn test_child_relationship() {
        let mut account = SObjectDescribe::new("Account");
        account.add_child_relationship(ChildRelationship::new(
            "Contacts",
            "Contact",
            "AccountId",
        ));

        let rel = account.get_child_relationship("Contacts").unwrap();
        assert_eq!(rel.child_object, "Contact");
        assert_eq!(rel.field, "AccountId");

        // Case-insensitive
        assert!(account.get_child_relationship("contacts").is_some());
    }
}

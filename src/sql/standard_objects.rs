//! Standard Salesforce object schema definitions
//!
//! This module provides schema definitions for common Salesforce standard objects
//! used in Sales Cloud, Service Cloud, and core platform functionality.

use super::schema::{
    ChildRelationship, FieldDescribe, SObjectDescribe, SalesforceFieldType, SalesforceSchema,
};

/// Create a schema with standard Salesforce Sales Cloud objects
pub fn create_sales_cloud_schema() -> SalesforceSchema {
    let mut schema = SalesforceSchema::new();

    // Add all standard objects
    schema.add_object(create_user());
    schema.add_object(create_account());
    schema.add_object(create_contact());
    schema.add_object(create_lead());
    schema.add_object(create_opportunity());
    schema.add_object(create_opportunity_contact_role());
    schema.add_object(create_opportunity_line_item());
    schema.add_object(create_product2());
    schema.add_object(create_pricebook2());
    schema.add_object(create_pricebook_entry());
    schema.add_object(create_case());
    schema.add_object(create_task());
    schema.add_object(create_event());
    schema.add_object(create_campaign());
    schema.add_object(create_campaign_member());
    schema.add_object(create_contract());
    schema.add_object(create_order());
    schema.add_object(create_order_item());
    schema.add_object(create_asset());
    schema.add_object(create_note());
    schema.add_object(create_attachment());

    schema
}

/// Add standard system fields to an object
fn add_system_fields(obj: &mut SObjectDescribe) {
    // Primary key
    obj.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));

    // Audit fields
    obj.add_field(
        FieldDescribe::new("CreatedById", SalesforceFieldType::Lookup)
            .with_reference("User")
            .with_relationship_name("CreatedBy"),
    );
    obj.add_field(FieldDescribe::new(
        "CreatedDate",
        SalesforceFieldType::DateTime,
    ));
    obj.add_field(
        FieldDescribe::new("LastModifiedById", SalesforceFieldType::Lookup)
            .with_reference("User")
            .with_relationship_name("LastModifiedBy"),
    );
    obj.add_field(FieldDescribe::new(
        "LastModifiedDate",
        SalesforceFieldType::DateTime,
    ));
    obj.add_field(FieldDescribe::new(
        "SystemModstamp",
        SalesforceFieldType::DateTime,
    ));

    // Soft delete
    obj.add_field(FieldDescribe::new(
        "IsDeleted",
        SalesforceFieldType::Boolean,
    ));
}

/// User object
fn create_user() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("User");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new("Username", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("FirstName", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("LastName", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Email", SalesforceFieldType::Email));
    obj.add_field(FieldDescribe::new("Phone", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new(
        "MobilePhone",
        SalesforceFieldType::Phone,
    ));
    obj.add_field(FieldDescribe::new("Title", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new(
        "Department",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "CompanyName",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new("IsActive", SalesforceFieldType::Boolean));
    obj.add_field(FieldDescribe::new(
        "UserType",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("ProfileId", SalesforceFieldType::Lookup));
    obj.add_field(FieldDescribe::new(
        "UserRoleId",
        SalesforceFieldType::Lookup,
    ));
    obj.add_field(
        FieldDescribe::new("ManagerId", SalesforceFieldType::Lookup)
            .with_reference("User")
            .with_relationship_name("Manager"),
    );
    obj.add_field(FieldDescribe::new(
        "TimeZoneSidKey",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "LocaleSidKey",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "LanguageLocaleKey",
        SalesforceFieldType::Picklist,
    ));

    obj
}

/// Account object - companies and organizations
fn create_account() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Account");
    add_system_fields(&mut obj);

    // Name and identification
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "AccountNumber",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new("Site", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Type", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "Industry",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("Rating", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "AccountSource",
        SalesforceFieldType::Picklist,
    ));

    // Financials
    obj.add_field(FieldDescribe::new(
        "AnnualRevenue",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberOfEmployees",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "Ownership",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "TickerSymbol",
        SalesforceFieldType::String,
    ));

    // Contact information
    obj.add_field(FieldDescribe::new("Phone", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new("Fax", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new("Website", SalesforceFieldType::Url));

    // Billing address
    obj.add_field(FieldDescribe::new(
        "BillingStreet",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingCity",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingState",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingPostalCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingCountry",
        SalesforceFieldType::String,
    ));

    // Shipping address
    obj.add_field(FieldDescribe::new(
        "ShippingStreet",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingCity",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingState",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingPostalCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingCountry",
        SalesforceFieldType::String,
    ));

    // Relationships
    obj.add_field(
        FieldDescribe::new("ParentId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Parent"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Description
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));

    // Activity tracking
    obj.add_field(FieldDescribe::new(
        "LastActivityDate",
        SalesforceFieldType::Date,
    ));

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new("Contacts", "Contact", "AccountId"));
    obj.add_child_relationship(ChildRelationship::new(
        "Opportunities",
        "Opportunity",
        "AccountId",
    ));
    obj.add_child_relationship(ChildRelationship::new("Cases", "Case", "AccountId"));
    obj.add_child_relationship(ChildRelationship::new("Tasks", "Task", "WhatId"));
    obj.add_child_relationship(ChildRelationship::new("Events", "Event", "WhatId"));
    obj.add_child_relationship(ChildRelationship::new("Contracts", "Contract", "AccountId"));
    obj.add_child_relationship(ChildRelationship::new("Orders", "Order", "AccountId"));
    obj.add_child_relationship(ChildRelationship::new("Assets", "Asset", "AccountId"));
    obj.add_child_relationship(ChildRelationship::new("Notes", "Note", "ParentId"));
    obj.add_child_relationship(ChildRelationship::new(
        "ChildAccounts",
        "Account",
        "ParentId",
    ));

    obj
}

/// Contact object - people associated with accounts
fn create_contact() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Contact");
    add_system_fields(&mut obj);

    // Name fields
    obj.add_field(FieldDescribe::new(
        "Salutation",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("FirstName", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("LastName", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Title", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new(
        "Department",
        SalesforceFieldType::String,
    ));

    // Contact information
    obj.add_field(FieldDescribe::new("Email", SalesforceFieldType::Email));
    obj.add_field(FieldDescribe::new("Phone", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new(
        "MobilePhone",
        SalesforceFieldType::Phone,
    ));
    obj.add_field(FieldDescribe::new("HomePhone", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new("OtherPhone", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new("Fax", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new(
        "AssistantName",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "AssistantPhone",
        SalesforceFieldType::Phone,
    ));

    // Mailing address
    obj.add_field(FieldDescribe::new(
        "MailingStreet",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "MailingCity",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "MailingState",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "MailingPostalCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "MailingCountry",
        SalesforceFieldType::String,
    ));

    // Other address
    obj.add_field(FieldDescribe::new(
        "OtherStreet",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new("OtherCity", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new(
        "OtherState",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "OtherPostalCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "OtherCountry",
        SalesforceFieldType::String,
    ));

    // Other fields
    obj.add_field(FieldDescribe::new("Birthdate", SalesforceFieldType::Date));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new(
        "LeadSource",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "HasOptedOutOfEmail",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "DoNotCall",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "HasOptedOutOfFax",
        SalesforceFieldType::Boolean,
    ));

    // Relationships
    obj.add_field(
        FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Account"),
    );
    obj.add_field(
        FieldDescribe::new("ReportsToId", SalesforceFieldType::Lookup)
            .with_reference("Contact")
            .with_relationship_name("ReportsTo"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Activity tracking
    obj.add_field(FieldDescribe::new(
        "LastActivityDate",
        SalesforceFieldType::Date,
    ));

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new("Cases", "Case", "ContactId"));
    obj.add_child_relationship(ChildRelationship::new(
        "OpportunityContactRoles",
        "OpportunityContactRole",
        "ContactId",
    ));
    obj.add_child_relationship(ChildRelationship::new(
        "CampaignMembers",
        "CampaignMember",
        "ContactId",
    ));

    obj
}

/// Lead object - potential customers
fn create_lead() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Lead");
    add_system_fields(&mut obj);

    // Name fields
    obj.add_field(FieldDescribe::new(
        "Salutation",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("FirstName", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("LastName", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Title", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Company", SalesforceFieldType::String).with_nillable(false));

    // Contact information
    obj.add_field(FieldDescribe::new("Email", SalesforceFieldType::Email));
    obj.add_field(FieldDescribe::new("Phone", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new(
        "MobilePhone",
        SalesforceFieldType::Phone,
    ));
    obj.add_field(FieldDescribe::new("Fax", SalesforceFieldType::Phone));
    obj.add_field(FieldDescribe::new("Website", SalesforceFieldType::Url));

    // Address
    obj.add_field(FieldDescribe::new("Street", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("City", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("State", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new(
        "PostalCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new("Country", SalesforceFieldType::String));

    // Lead details
    obj.add_field(FieldDescribe::new(
        "LeadSource",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "Industry",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("Rating", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "AnnualRevenue",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberOfEmployees",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));

    // Conversion tracking
    obj.add_field(FieldDescribe::new(
        "IsConverted",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "ConvertedDate",
        SalesforceFieldType::Date,
    ));
    obj.add_field(
        FieldDescribe::new("ConvertedAccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("ConvertedAccount"),
    );
    obj.add_field(
        FieldDescribe::new("ConvertedContactId", SalesforceFieldType::Lookup)
            .with_reference("Contact")
            .with_relationship_name("ConvertedContact"),
    );
    obj.add_field(
        FieldDescribe::new("ConvertedOpportunityId", SalesforceFieldType::Lookup)
            .with_reference("Opportunity")
            .with_relationship_name("ConvertedOpportunity"),
    );

    // Opt-out fields
    obj.add_field(FieldDescribe::new(
        "HasOptedOutOfEmail",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "DoNotCall",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "HasOptedOutOfFax",
        SalesforceFieldType::Boolean,
    ));

    // Relationships
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Activity tracking
    obj.add_field(FieldDescribe::new(
        "LastActivityDate",
        SalesforceFieldType::Date,
    ));

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new(
        "CampaignMembers",
        "CampaignMember",
        "LeadId",
    ));

    obj
}

/// Opportunity object - sales deals
fn create_opportunity() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Opportunity");
    add_system_fields(&mut obj);

    // Core fields
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(
        FieldDescribe::new("StageName", SalesforceFieldType::Picklist).with_nillable(false),
    );
    obj.add_field(FieldDescribe::new("CloseDate", SalesforceFieldType::Date).with_nillable(false));
    obj.add_field(FieldDescribe::new("Amount", SalesforceFieldType::Currency));
    obj.add_field(FieldDescribe::new(
        "Probability",
        SalesforceFieldType::Percent,
    ));
    obj.add_field(FieldDescribe::new(
        "ExpectedRevenue",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "TotalOpportunityQuantity",
        SalesforceFieldType::Double,
    ));

    // Type and source
    obj.add_field(FieldDescribe::new("Type", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "LeadSource",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "ForecastCategory",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "ForecastCategoryName",
        SalesforceFieldType::Picklist,
    ));

    // Status
    obj.add_field(FieldDescribe::new("IsClosed", SalesforceFieldType::Boolean));
    obj.add_field(FieldDescribe::new("IsWon", SalesforceFieldType::Boolean));
    obj.add_field(FieldDescribe::new(
        "FiscalQuarter",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "FiscalYear",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new("Fiscal", SalesforceFieldType::String));

    // Description
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new("NextStep", SalesforceFieldType::String));

    // Relationships
    obj.add_field(
        FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Account"),
    );
    obj.add_field(
        FieldDescribe::new("CampaignId", SalesforceFieldType::Lookup)
            .with_reference("Campaign")
            .with_relationship_name("Campaign"),
    );
    obj.add_field(
        FieldDescribe::new("Pricebook2Id", SalesforceFieldType::Lookup)
            .with_reference("Pricebook2")
            .with_relationship_name("Pricebook2"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Activity tracking
    obj.add_field(FieldDescribe::new(
        "LastActivityDate",
        SalesforceFieldType::Date,
    ));

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new(
        "OpportunityContactRoles",
        "OpportunityContactRole",
        "OpportunityId",
    ));
    obj.add_child_relationship(ChildRelationship::new(
        "OpportunityLineItems",
        "OpportunityLineItem",
        "OpportunityId",
    ));

    obj
}

/// OpportunityContactRole - junction between Opportunity and Contact
fn create_opportunity_contact_role() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("OpportunityContactRole");
    add_system_fields(&mut obj);

    obj.add_field(
        FieldDescribe::new("OpportunityId", SalesforceFieldType::Lookup)
            .with_reference("Opportunity")
            .with_relationship_name("Opportunity"),
    );
    obj.add_field(
        FieldDescribe::new("ContactId", SalesforceFieldType::Lookup)
            .with_reference("Contact")
            .with_relationship_name("Contact"),
    );
    obj.add_field(FieldDescribe::new("Role", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "IsPrimary",
        SalesforceFieldType::Boolean,
    ));

    obj
}

/// OpportunityLineItem - products on an opportunity
fn create_opportunity_line_item() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("OpportunityLineItem");
    add_system_fields(&mut obj);

    obj.add_field(
        FieldDescribe::new("OpportunityId", SalesforceFieldType::Lookup)
            .with_reference("Opportunity")
            .with_relationship_name("Opportunity"),
    );
    obj.add_field(
        FieldDescribe::new("PricebookEntryId", SalesforceFieldType::Lookup)
            .with_reference("PricebookEntry")
            .with_relationship_name("PricebookEntry"),
    );
    obj.add_field(
        FieldDescribe::new("Product2Id", SalesforceFieldType::Lookup)
            .with_reference("Product2")
            .with_relationship_name("Product2"),
    );
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Quantity", SalesforceFieldType::Double).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "UnitPrice",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "TotalPrice",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "ListPrice",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new("Discount", SalesforceFieldType::Percent));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new("ServiceDate", SalesforceFieldType::Date));
    obj.add_field(FieldDescribe::new(
        "ProductCode",
        SalesforceFieldType::String,
    ));

    obj
}

/// Product2 - products in the catalog
fn create_product2() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Product2");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "ProductCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new("Family", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new("IsActive", SalesforceFieldType::Boolean));
    obj.add_field(FieldDescribe::new(
        "ExternalId",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "QuantityUnitOfMeasure",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "StockKeepingUnit",
        SalesforceFieldType::String,
    ));

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new(
        "PricebookEntries",
        "PricebookEntry",
        "Product2Id",
    ));

    obj
}

/// Pricebook2 - price books
fn create_pricebook2() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Pricebook2");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new("IsActive", SalesforceFieldType::Boolean));
    obj.add_field(FieldDescribe::new(
        "IsStandard",
        SalesforceFieldType::Boolean,
    ));

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new(
        "PricebookEntries",
        "PricebookEntry",
        "Pricebook2Id",
    ));

    obj
}

/// PricebookEntry - products in a pricebook
fn create_pricebook_entry() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("PricebookEntry");
    add_system_fields(&mut obj);

    obj.add_field(
        FieldDescribe::new("Pricebook2Id", SalesforceFieldType::Lookup)
            .with_reference("Pricebook2")
            .with_relationship_name("Pricebook2"),
    );
    obj.add_field(
        FieldDescribe::new("Product2Id", SalesforceFieldType::Lookup)
            .with_reference("Product2")
            .with_relationship_name("Product2"),
    );
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    obj.add_field(
        FieldDescribe::new("UnitPrice", SalesforceFieldType::Currency).with_nillable(false),
    );
    obj.add_field(FieldDescribe::new("IsActive", SalesforceFieldType::Boolean));
    obj.add_field(FieldDescribe::new(
        "UseStandardPrice",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "ProductCode",
        SalesforceFieldType::String,
    ));

    obj
}

/// Case object - customer service cases
fn create_case() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Case");
    add_system_fields(&mut obj);

    // Case identification
    obj.add_field(FieldDescribe::new("CaseNumber", SalesforceFieldType::Auto));
    obj.add_field(FieldDescribe::new("Subject", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));

    // Status fields
    obj.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "Priority",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("Origin", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new("Reason", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new("Type", SalesforceFieldType::Picklist));

    // Flags
    obj.add_field(FieldDescribe::new("IsClosed", SalesforceFieldType::Boolean));
    obj.add_field(FieldDescribe::new(
        "IsEscalated",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "IsClosedOnCreate",
        SalesforceFieldType::Boolean,
    ));

    // Dates
    obj.add_field(FieldDescribe::new(
        "ClosedDate",
        SalesforceFieldType::DateTime,
    ));

    // Contact information
    obj.add_field(FieldDescribe::new(
        "SuppliedName",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "SuppliedEmail",
        SalesforceFieldType::Email,
    ));
    obj.add_field(FieldDescribe::new(
        "SuppliedPhone",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "SuppliedCompany",
        SalesforceFieldType::String,
    ));

    // Relationships
    obj.add_field(
        FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Account"),
    );
    obj.add_field(
        FieldDescribe::new("ContactId", SalesforceFieldType::Lookup)
            .with_reference("Contact")
            .with_relationship_name("Contact"),
    );
    obj.add_field(
        FieldDescribe::new("ParentId", SalesforceFieldType::Lookup)
            .with_reference("Case")
            .with_relationship_name("Parent"),
    );
    obj.add_field(
        FieldDescribe::new("AssetId", SalesforceFieldType::Lookup)
            .with_reference("Asset")
            .with_relationship_name("Asset"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Comments
    obj.add_field(FieldDescribe::new(
        "Comments",
        SalesforceFieldType::LongTextArea,
    ));

    obj
}

/// Task object - activities/to-do items
fn create_task() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Task");
    add_system_fields(&mut obj);

    // Core fields
    obj.add_field(FieldDescribe::new("Subject", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "Priority",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("Type", SalesforceFieldType::Picklist));

    // Dates
    obj.add_field(FieldDescribe::new(
        "ActivityDate",
        SalesforceFieldType::Date,
    ));
    obj.add_field(FieldDescribe::new(
        "CompletedDateTime",
        SalesforceFieldType::DateTime,
    ));

    // Flags
    obj.add_field(FieldDescribe::new("IsClosed", SalesforceFieldType::Boolean));
    obj.add_field(FieldDescribe::new(
        "IsHighPriority",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "IsRecurrence",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "IsReminderSet",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "ReminderDateTime",
        SalesforceFieldType::DateTime,
    ));

    // Polymorphic relationships (What = Account, Opportunity, etc.)
    obj.add_field(
        FieldDescribe::new("WhatId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec![
                "Account".to_string(),
                "Opportunity".to_string(),
                "Campaign".to_string(),
                "Case".to_string(),
                "Contract".to_string(),
            ])
            .with_relationship_name("What"),
    );
    obj.add_field(
        FieldDescribe::new("WhoId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["Contact".to_string(), "Lead".to_string()])
            .with_relationship_name("Who"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Call-related fields
    obj.add_field(FieldDescribe::new(
        "CallType",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "CallDurationInSeconds",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "CallObject",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "CallDisposition",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "CallResult",
        SalesforceFieldType::String,
    ));

    obj
}

/// Event object - calendar events/meetings
fn create_event() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Event");
    add_system_fields(&mut obj);

    // Core fields
    obj.add_field(FieldDescribe::new("Subject", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new("Location", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Type", SalesforceFieldType::Picklist));

    // Date/time fields
    obj.add_field(FieldDescribe::new(
        "StartDateTime",
        SalesforceFieldType::DateTime,
    ));
    obj.add_field(FieldDescribe::new(
        "EndDateTime",
        SalesforceFieldType::DateTime,
    ));
    obj.add_field(FieldDescribe::new(
        "ActivityDate",
        SalesforceFieldType::Date,
    ));
    obj.add_field(FieldDescribe::new(
        "ActivityDateTime",
        SalesforceFieldType::DateTime,
    ));
    obj.add_field(FieldDescribe::new(
        "DurationInMinutes",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "IsAllDayEvent",
        SalesforceFieldType::Boolean,
    ));

    // Flags
    obj.add_field(FieldDescribe::new(
        "IsPrivate",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new("ShowAs", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "IsRecurrence",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "IsReminderSet",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "ReminderDateTime",
        SalesforceFieldType::DateTime,
    ));

    // Polymorphic relationships
    obj.add_field(
        FieldDescribe::new("WhatId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec![
                "Account".to_string(),
                "Opportunity".to_string(),
                "Campaign".to_string(),
                "Case".to_string(),
                "Contract".to_string(),
            ])
            .with_relationship_name("What"),
    );
    obj.add_field(
        FieldDescribe::new("WhoId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["Contact".to_string(), "Lead".to_string()])
            .with_relationship_name("Who"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    obj
}

/// Campaign object - marketing campaigns
fn create_campaign() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Campaign");
    add_system_fields(&mut obj);

    // Core fields
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new("Type", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist));

    // Dates
    obj.add_field(FieldDescribe::new("StartDate", SalesforceFieldType::Date));
    obj.add_field(FieldDescribe::new("EndDate", SalesforceFieldType::Date));

    // Budget and costs
    obj.add_field(FieldDescribe::new(
        "BudgetedCost",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "ActualCost",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "ExpectedRevenue",
        SalesforceFieldType::Currency,
    ));

    // Response metrics
    obj.add_field(FieldDescribe::new(
        "ExpectedResponse",
        SalesforceFieldType::Percent,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberSent",
        SalesforceFieldType::Double,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberOfLeads",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberOfConvertedLeads",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberOfContacts",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberOfResponses",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberOfOpportunities",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "NumberOfWonOpportunities",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "AmountAllOpportunities",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "AmountWonOpportunities",
        SalesforceFieldType::Currency,
    ));

    // Flags
    obj.add_field(FieldDescribe::new("IsActive", SalesforceFieldType::Boolean));

    // Relationships
    obj.add_field(
        FieldDescribe::new("ParentId", SalesforceFieldType::Lookup)
            .with_reference("Campaign")
            .with_relationship_name("Parent"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new(
        "CampaignMembers",
        "CampaignMember",
        "CampaignId",
    ));
    obj.add_child_relationship(ChildRelationship::new(
        "Opportunities",
        "Opportunity",
        "CampaignId",
    ));
    obj.add_child_relationship(ChildRelationship::new(
        "ChildCampaigns",
        "Campaign",
        "ParentId",
    ));

    obj
}

/// CampaignMember - junction between Campaign and Lead/Contact
fn create_campaign_member() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("CampaignMember");
    add_system_fields(&mut obj);

    obj.add_field(
        FieldDescribe::new("CampaignId", SalesforceFieldType::Lookup)
            .with_reference("Campaign")
            .with_relationship_name("Campaign"),
    );
    obj.add_field(
        FieldDescribe::new("LeadId", SalesforceFieldType::Lookup)
            .with_reference("Lead")
            .with_relationship_name("Lead"),
    );
    obj.add_field(
        FieldDescribe::new("ContactId", SalesforceFieldType::Lookup)
            .with_reference("Contact")
            .with_relationship_name("Contact"),
    );
    obj.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "HasResponded",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(FieldDescribe::new(
        "FirstRespondedDate",
        SalesforceFieldType::Date,
    ));
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new("Title", SalesforceFieldType::String));
    obj.add_field(FieldDescribe::new(
        "CompanyOrAccount",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "LeadOrContactId",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "LeadOrContactOwnerId",
        SalesforceFieldType::String,
    ));

    obj
}

/// Contract object
fn create_contract() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Contract");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new(
        "ContractNumber",
        SalesforceFieldType::Auto,
    ));
    obj.add_field(
        FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Account"),
    );
    obj.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "StatusCode",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new("StartDate", SalesforceFieldType::Date));
    obj.add_field(FieldDescribe::new("EndDate", SalesforceFieldType::Date));
    obj.add_field(FieldDescribe::new(
        "ContractTerm",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "OwnerExpirationNotice",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "CompanySignedDate",
        SalesforceFieldType::Date,
    ));
    obj.add_field(
        FieldDescribe::new("CompanySignedId", SalesforceFieldType::Lookup)
            .with_reference("User")
            .with_relationship_name("CompanySigned"),
    );
    obj.add_field(FieldDescribe::new(
        "CustomerSignedDate",
        SalesforceFieldType::Date,
    ));
    obj.add_field(
        FieldDescribe::new("CustomerSignedId", SalesforceFieldType::Lookup)
            .with_reference("Contact")
            .with_relationship_name("CustomerSigned"),
    );
    obj.add_field(FieldDescribe::new(
        "CustomerSignedTitle",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "SpecialTerms",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new(
        "ActivatedDate",
        SalesforceFieldType::DateTime,
    ));
    obj.add_field(
        FieldDescribe::new("ActivatedById", SalesforceFieldType::Lookup)
            .with_reference("User")
            .with_relationship_name("ActivatedBy"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Address
    obj.add_field(FieldDescribe::new(
        "BillingStreet",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingCity",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingState",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingPostalCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingCountry",
        SalesforceFieldType::String,
    ));

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new("Orders", "Order", "ContractId"));

    obj
}

/// Order object
fn create_order() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Order");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new("OrderNumber", SalesforceFieldType::Auto));
    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    obj.add_field(
        FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Account"),
    );
    obj.add_field(
        FieldDescribe::new("ContractId", SalesforceFieldType::Lookup)
            .with_reference("Contract")
            .with_relationship_name("Contract"),
    );
    obj.add_field(
        FieldDescribe::new("Pricebook2Id", SalesforceFieldType::Lookup)
            .with_reference("Pricebook2")
            .with_relationship_name("Pricebook2"),
    );
    obj.add_field(
        FieldDescribe::new("OpportunityId", SalesforceFieldType::Lookup)
            .with_reference("Opportunity")
            .with_relationship_name("Opportunity"),
    );
    obj.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "StatusCode",
        SalesforceFieldType::Picklist,
    ));
    obj.add_field(FieldDescribe::new(
        "EffectiveDate",
        SalesforceFieldType::Date,
    ));
    obj.add_field(FieldDescribe::new("EndDate", SalesforceFieldType::Date));
    obj.add_field(FieldDescribe::new("Type", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new(
        "TotalAmount",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));

    // Address
    obj.add_field(FieldDescribe::new(
        "BillingStreet",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingCity",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingState",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingPostalCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "BillingCountry",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingStreet",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingCity",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingState",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingPostalCode",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "ShippingCountry",
        SalesforceFieldType::String,
    ));

    obj.add_field(FieldDescribe::new(
        "ActivatedDate",
        SalesforceFieldType::DateTime,
    ));
    obj.add_field(
        FieldDescribe::new("ActivatedById", SalesforceFieldType::Lookup)
            .with_reference("User")
            .with_relationship_name("ActivatedBy"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new("OrderItems", "OrderItem", "OrderId"));

    obj
}

/// OrderItem object
fn create_order_item() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("OrderItem");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new(
        "OrderItemNumber",
        SalesforceFieldType::Auto,
    ));
    obj.add_field(
        FieldDescribe::new("OrderId", SalesforceFieldType::Lookup)
            .with_reference("Order")
            .with_relationship_name("Order"),
    );
    obj.add_field(
        FieldDescribe::new("PricebookEntryId", SalesforceFieldType::Lookup)
            .with_reference("PricebookEntry")
            .with_relationship_name("PricebookEntry"),
    );
    obj.add_field(
        FieldDescribe::new("Product2Id", SalesforceFieldType::Lookup)
            .with_reference("Product2")
            .with_relationship_name("Product2"),
    );
    obj.add_field(FieldDescribe::new("Quantity", SalesforceFieldType::Double).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "UnitPrice",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "TotalPrice",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "ListPrice",
        SalesforceFieldType::Currency,
    ));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new("ServiceDate", SalesforceFieldType::Date));
    obj.add_field(FieldDescribe::new("EndDate", SalesforceFieldType::Date));

    obj
}

/// Asset object - products owned by customers
fn create_asset() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Asset");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "SerialNumber",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist));
    obj.add_field(FieldDescribe::new("Quantity", SalesforceFieldType::Double));
    obj.add_field(FieldDescribe::new("Price", SalesforceFieldType::Currency));
    obj.add_field(FieldDescribe::new(
        "PurchaseDate",
        SalesforceFieldType::Date,
    ));
    obj.add_field(FieldDescribe::new("InstallDate", SalesforceFieldType::Date));
    obj.add_field(FieldDescribe::new(
        "UsageEndDate",
        SalesforceFieldType::Date,
    ));

    obj.add_field(
        FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Account"),
    );
    obj.add_field(
        FieldDescribe::new("ContactId", SalesforceFieldType::Lookup)
            .with_reference("Contact")
            .with_relationship_name("Contact"),
    );
    obj.add_field(
        FieldDescribe::new("Product2Id", SalesforceFieldType::Lookup)
            .with_reference("Product2")
            .with_relationship_name("Product2"),
    );
    obj.add_field(
        FieldDescribe::new("ParentId", SalesforceFieldType::Lookup)
            .with_reference("Asset")
            .with_relationship_name("Parent"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    // Child relationships
    obj.add_child_relationship(ChildRelationship::new("Cases", "Case", "AssetId"));
    obj.add_child_relationship(ChildRelationship::new("ChildAssets", "Asset", "ParentId"));

    obj
}

/// Note object - attached notes
fn create_note() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Note");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new("Title", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "Body",
        SalesforceFieldType::LongTextArea,
    ));
    obj.add_field(FieldDescribe::new(
        "IsPrivate",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(
        FieldDescribe::new("ParentId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec![
                "Account".to_string(),
                "Contact".to_string(),
                "Opportunity".to_string(),
                "Lead".to_string(),
                "Case".to_string(),
            ])
            .with_relationship_name("Parent"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    obj
}

/// Attachment object - file attachments
fn create_attachment() -> SObjectDescribe {
    let mut obj = SObjectDescribe::new("Attachment");
    add_system_fields(&mut obj);

    obj.add_field(FieldDescribe::new("Name", SalesforceFieldType::String).with_nillable(false));
    obj.add_field(FieldDescribe::new(
        "Body",
        SalesforceFieldType::LongTextArea,
    )); // Actually Base64
    obj.add_field(FieldDescribe::new(
        "BodyLength",
        SalesforceFieldType::Integer,
    ));
    obj.add_field(FieldDescribe::new(
        "ContentType",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "Description",
        SalesforceFieldType::String,
    ));
    obj.add_field(FieldDescribe::new(
        "IsPrivate",
        SalesforceFieldType::Boolean,
    ));
    obj.add_field(
        FieldDescribe::new("ParentId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec![
                "Account".to_string(),
                "Contact".to_string(),
                "Opportunity".to_string(),
                "Lead".to_string(),
                "Case".to_string(),
            ])
            .with_relationship_name("Parent"),
    );
    obj.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );

    obj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sales_cloud_schema_has_all_objects() {
        let schema = create_sales_cloud_schema();

        // Verify all expected objects exist
        assert!(schema.get_object("User").is_some());
        assert!(schema.get_object("Account").is_some());
        assert!(schema.get_object("Contact").is_some());
        assert!(schema.get_object("Lead").is_some());
        assert!(schema.get_object("Opportunity").is_some());
        assert!(schema.get_object("OpportunityContactRole").is_some());
        assert!(schema.get_object("OpportunityLineItem").is_some());
        assert!(schema.get_object("Product2").is_some());
        assert!(schema.get_object("Pricebook2").is_some());
        assert!(schema.get_object("PricebookEntry").is_some());
        assert!(schema.get_object("Case").is_some());
        assert!(schema.get_object("Task").is_some());
        assert!(schema.get_object("Event").is_some());
        assert!(schema.get_object("Campaign").is_some());
        assert!(schema.get_object("CampaignMember").is_some());
        assert!(schema.get_object("Contract").is_some());
        assert!(schema.get_object("Order").is_some());
        assert!(schema.get_object("OrderItem").is_some());
        assert!(schema.get_object("Asset").is_some());
        assert!(schema.get_object("Note").is_some());
        assert!(schema.get_object("Attachment").is_some());
    }

    #[test]
    fn test_account_has_expected_fields() {
        let schema = create_sales_cloud_schema();
        let account = schema.get_object("Account").unwrap();

        // Core fields
        assert!(account.get_field("Id").is_some());
        assert!(account.get_field("Name").is_some());
        assert!(account.get_field("Industry").is_some());
        assert!(account.get_field("AnnualRevenue").is_some());

        // Relationships
        assert!(account.get_field("ParentId").is_some());
        assert!(account.get_field("OwnerId").is_some());

        // Child relationships
        assert!(account.get_child_relationship("Contacts").is_some());
        assert!(account.get_child_relationship("Opportunities").is_some());
        assert!(account.get_child_relationship("Cases").is_some());
    }

    #[test]
    fn test_opportunity_has_expected_fields() {
        let schema = create_sales_cloud_schema();
        let opp = schema.get_object("Opportunity").unwrap();

        // Core fields
        assert!(opp.get_field("Name").is_some());
        assert!(opp.get_field("StageName").is_some());
        assert!(opp.get_field("CloseDate").is_some());
        assert!(opp.get_field("Amount").is_some());
        assert!(opp.get_field("Probability").is_some());

        // Relationships
        assert!(opp.get_field("AccountId").is_some());
        let account_field = opp.get_field("AccountId").unwrap();
        assert_eq!(account_field.relationship_name, Some("Account".to_string()));
    }

    #[test]
    fn test_task_has_polymorphic_fields() {
        let schema = create_sales_cloud_schema();
        let task = schema.get_object("Task").unwrap();

        let what_field = task.get_field("WhatId").unwrap();
        assert!(what_field.is_polymorphic);
        assert!(what_field
            .reference_to
            .as_ref()
            .unwrap()
            .contains(&"Account".to_string()));
        assert!(what_field
            .reference_to
            .as_ref()
            .unwrap()
            .contains(&"Opportunity".to_string()));

        let who_field = task.get_field("WhoId").unwrap();
        assert!(who_field.is_polymorphic);
        assert!(who_field
            .reference_to
            .as_ref()
            .unwrap()
            .contains(&"Contact".to_string()));
        assert!(who_field
            .reference_to
            .as_ref()
            .unwrap()
            .contains(&"Lead".to_string()));
    }
}

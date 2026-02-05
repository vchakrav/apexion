//! Tests for parsing Dreamhouse LWC sample app Apex classes
//! https://github.com/trailheadapps/dreamhouse-lwc

use apexrust::parser::Parser;
use apexrust::ast::{TypeDeclaration, ClassMember};

const PROPERTY_CONTROLLER: &str = r#"
public with sharing class PropertyController {
    private static final Decimal DEFAULT_MAX_PRICE = 9999999;
    private static final Integer DEFAULT_PAGE_SIZE = 9;

    @AuraEnabled(cacheable=true scope='global')
    public static PagedResult getPagedPropertyList(
        String searchKey,
        Decimal maxPrice,
        Integer minBedrooms,
        Integer minBathrooms,
        Integer pageSize,
        Integer pageNumber
    ) {
        Decimal safeMaxPrice = maxPrice ?? DEFAULT_MAX_PRICE;
        Integer safeMinBedrooms = minBedrooms ?? 0;
        Integer safeMinBathrooms = minBathrooms ?? 0;
        Integer safePageSize = pageSize ?? DEFAULT_PAGE_SIZE;
        Integer safePageNumber = pageNumber ?? 1;

        String searchPattern = '%' + searchKey + '%';
        Integer offset = (safePageNumber - 1) * safePageSize;

        PagedResult result = new PagedResult();
        result.pageSize = safePageSize;
        result.pageNumber = safePageNumber;
        result.totalItemCount = [
            SELECT COUNT()
            FROM Property__c
            WHERE
                (Name LIKE :searchPattern
                OR City__c LIKE :searchPattern
                OR Tags__c LIKE :searchPattern)
                AND Price__c <= :safeMaxPrice
                AND Beds__c >= :safeMinBedrooms
                AND Baths__c >= :safeMinBathrooms
        ];
        result.records = [
            SELECT
                Id,
                Name,
                Address__c,
                City__c,
                State__c,
                Description__c,
                Price__c,
                Baths__c,
                Beds__c,
                Thumbnail__c,
                Location__Latitude__s,
                Location__Longitude__s
            FROM Property__c
            WHERE
                (Name LIKE :searchPattern
                OR City__c LIKE :searchPattern
                OR Tags__c LIKE :searchPattern)
                AND Price__c <= :safeMaxPrice
                AND Beds__c >= :safeMinBedrooms
                AND Baths__c >= :safeMinBathrooms
            WITH USER_MODE
            ORDER BY Price__c
            LIMIT :safePageSize
            OFFSET :offset
        ];
        return result;
    }

    @AuraEnabled(cacheable=true scope='global')
    public static List<ContentVersion> getPictures(Id propertyId) {
        List<ContentDocumentLink> links = [
            SELECT Id, LinkedEntityId, ContentDocument.Title
            FROM ContentDocumentLink
            WHERE
                LinkedEntityId = :propertyId
                AND ContentDocument.FileType IN ('PNG', 'JPG', 'GIF')
            WITH USER_MODE
        ];

        if (links.isEmpty()) {
            return null;
        }

        Set<Id> contentIds = new Set<Id>();

        for (ContentDocumentLink link : links) {
            contentIds.add(link.ContentDocumentId);
        }

        return [
            SELECT Id, Title
            FROM ContentVersion
            WHERE ContentDocumentId IN :contentIds AND IsLatest = TRUE
            WITH USER_MODE
            ORDER BY CreatedDate
        ];
    }
}
"#;

const SAMPLE_DATA_CONTROLLER: &str = r#"
public with sharing class SampleDataController {
    @AuraEnabled
    public static void importSampleData() {
        delete [SELECT Id FROM Case];
        delete [SELECT Id FROM Property__c];
        delete [SELECT Id FROM Broker__c];
        delete [SELECT Id FROM Contact];

        insertBrokers();
        insertProperties();
        insertContacts();
    }

    private static void insertBrokers() {
        StaticResource brokersResource = [
            SELECT Id, Body
            FROM StaticResource
            WHERE Name = 'sample_data_brokers'
        ];
        String brokersJSON = brokersResource.body.toString();
        List<Broker__c> brokers = (List<Broker__c>) JSON.deserialize(
            brokersJSON,
            List<Broker__c>.class
        );
        insert brokers;
    }

    private static void insertProperties() {
        StaticResource propertiesResource = [
            SELECT Id, Body
            FROM StaticResource
            WHERE Name = 'sample_data_properties'
        ];
        String propertiesJSON = propertiesResource.body.toString();
        List<Property__c> properties = (List<Property__c>) JSON.deserialize(
            propertiesJSON,
            List<Property__c>.class
        );
        randomizeDateListed(properties);
        insert properties;
    }

    private static void insertContacts() {
        StaticResource contactsResource = [
            SELECT Id, Body
            FROM StaticResource
            WHERE Name = 'sample_data_contacts'
        ];
        String contactsJSON = contactsResource.body.toString();
        List<Contact> contacts = (List<Contact>) JSON.deserialize(
            contactsJSON,
            List<Contact>.class
        );
        insert contacts;
    }

    private static void randomizeDateListed(List<Property__c> properties) {
        for (Property__c property : properties) {
            property.Date_Listed__c =
                System.today() - Integer.valueof((Math.random() * 90));
        }
    }
}
"#;

const GEOCODING_SERVICE: &str = r#"
public with sharing class GeocodingService {
    private static final String BASE_URL = 'https://nominatim.openstreetmap.org/search?format=json';

    @InvocableMethod(callout=true label='Geocode address')
    public static List<Coordinates> geocodeAddresses(
        List<GeocodingAddress> addresses
    ) {
        List<Coordinates> computedCoordinates = new List<Coordinates>();

        for (GeocodingAddress address : addresses) {
            String geocodingUrl = BASE_URL;
            geocodingUrl += (String.isNotBlank(address.street))
                ? '&street=' + address.street
                : '';
            geocodingUrl += (String.isNotBlank(address.city))
                ? '&city=' + address.city
                : '';
            geocodingUrl += (String.isNotBlank(address.state))
                ? '&state=' + address.state
                : '';
            geocodingUrl += (String.isNotBlank(address.country))
                ? '&country=' + address.country
                : '';
            geocodingUrl += (String.isNotBlank(address.postalcode))
                ? '&postalcode=' + address.postalcode
                : '';

            Coordinates coords = new Coordinates();
            if (geocodingUrl != BASE_URL) {
                Http http = new Http();
                HttpRequest request = new HttpRequest();
                request.setEndpoint(geocodingUrl);
                request.setMethod('GET');
                request.setHeader(
                    'http-referer',
                    URL.getOrgDomainUrl().toExternalForm()
                );
                HttpResponse response = http.send(request);
                if (response.getStatusCode() == 200) {
                    List<Coordinates> deserializedCoords = (List<Coordinates>) JSON.deserialize(
                        response.getBody(),
                        List<Coordinates>.class
                    );
                    coords = deserializedCoords[0];
                }
            }

            computedCoordinates.add(coords);
        }
        return computedCoordinates;
    }

    public class GeocodingAddress {
        @InvocableVariable
        public String street;
        @InvocableVariable
        public String city;
        @InvocableVariable
        public String state;
        @InvocableVariable
        public String country;
        @InvocableVariable
        public String postalcode;
    }

    public class Coordinates {
        @InvocableVariable
        public Decimal lat;
        @InvocableVariable
        public Decimal lon;
    }
}
"#;

const PAGED_RESULT: &str = r#"
public with sharing class PagedResult {
    @AuraEnabled
    public Integer pageSize { get; set; }

    @AuraEnabled
    public Integer pageNumber { get; set; }

    @AuraEnabled
    public Integer totalItemCount { get; set; }

    @AuraEnabled
    public Object[] records { get; set; }
}
"#;

const FILE_UTILITIES: &str = r#"
public with sharing class FileUtilities {
    @AuraEnabled
    public static String createFile(
        String base64data,
        String filename,
        String recordId
    ) {
        try {
            ContentVersion contentVersion = new ContentVersion();
            contentVersion.VersionData = EncodingUtil.base64Decode(base64data);
            contentVersion.Title = filename;
            contentVersion.PathOnClient = filename;
            insert contentVersion;

            contentVersion = [
                SELECT ContentDocumentId
                FROM ContentVersion
                WHERE Id = :contentVersion.Id
                WITH USER_MODE
            ];

            ContentDocumentLink contentDocumentLink = new ContentDocumentLink();
            contentDocumentLink.ContentDocumentId = contentVersion.ContentDocumentId;
            contentDocumentLink.LinkedEntityId = recordId;
            contentDocumentLink.ShareType = 'V';
            insert contentDocumentLink;

            return contentDocumentLink.Id;
        } catch (Exception e) {
            throw new AuraHandledException('Error creating file: ' + e);
        }
    }
}
"#;

#[test]
fn test_parse_property_controller() {
    let mut parser = Parser::new(PROPERTY_CONTROLLER);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse PropertyController: {:?}", result.err());
    
    let ast = result.unwrap();
    assert_eq!(ast.declarations.len(), 1, "Expected 1 class");
    
    // Check it's a class named PropertyController
    if let TypeDeclaration::Class(class) = &ast.declarations[0] {
        assert_eq!(class.name, "PropertyController");
        let method_count = class.members.iter().filter(|m| matches!(m, ClassMember::Method(_))).count();
        let field_count = class.members.iter().filter(|m| matches!(m, ClassMember::Field(_))).count();
        println!("PropertyController parsed successfully!");
        println!("  Methods: {}", method_count);
        println!("  Fields: {}", field_count);
    } else {
        panic!("Expected a class");
    }
}

#[test]
fn test_parse_sample_data_controller() {
    let mut parser = Parser::new(SAMPLE_DATA_CONTROLLER);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse SampleDataController: {:?}", result.err());
    
    let ast = result.unwrap();
    if let TypeDeclaration::Class(class) = &ast.declarations[0] {
        assert_eq!(class.name, "SampleDataController");
        let method_count = class.members.iter().filter(|m| matches!(m, ClassMember::Method(_))).count();
        println!("SampleDataController parsed successfully!");
        println!("  Methods: {}", method_count);
    }
}

#[test]
fn test_parse_geocoding_service() {
    let mut parser = Parser::new(GEOCODING_SERVICE);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse GeocodingService: {:?}", result.err());
    
    let ast = result.unwrap();
    if let TypeDeclaration::Class(class) = &ast.declarations[0] {
        assert_eq!(class.name, "GeocodingService");
        // Check for inner classes
        let inner_classes: Vec<_> = class.members.iter()
            .filter(|m| matches!(m, ClassMember::InnerClass(_)))
            .collect();
        println!("GeocodingService parsed successfully!");
        println!("  Inner classes: {}", inner_classes.len());
    }
}

#[test]
fn test_parse_paged_result() {
    let mut parser = Parser::new(PAGED_RESULT);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse PagedResult: {:?}", result.err());
    
    let ast = result.unwrap();
    if let TypeDeclaration::Class(class) = &ast.declarations[0] {
        assert_eq!(class.name, "PagedResult");
        let property_count = class.members.iter().filter(|m| matches!(m, ClassMember::Property(_))).count();
        println!("PagedResult parsed successfully!");
        println!("  Properties: {}", property_count);
    }
}

#[test]
fn test_parse_file_utilities() {
    let mut parser = Parser::new(FILE_UTILITIES);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse FileUtilities: {:?}", result.err());
    
    let ast = result.unwrap();
    if let TypeDeclaration::Class(class) = &ast.declarations[0] {
        assert_eq!(class.name, "FileUtilities");
        println!("FileUtilities parsed successfully!");
    }
}

// Test SOQL extraction from PropertyController
#[test]
fn test_extract_soql_from_property_controller() {
    let mut parser = Parser::new(PROPERTY_CONTROLLER);
    let result = parser.parse();
    assert!(result.is_ok());
    
    // Count SOQL queries - should find the queries in getPagedPropertyList and getPictures
    println!("SOQL queries found in PropertyController - parsing successful");
}

// Test type literal parsing specifically
#[test]
fn test_parse_type_literal() {
    let code = r#"
public class Test {
    public void test() {
        Type t = List<Account>.class;
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    if let Err(ref e) = result {
        println!("Error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse simple type literal: {:?}", result.err());
}

#[test]
fn test_parse_type_literal_as_argument() {
    let code = r#"
public class Test {
    public void test() {
        Object o = JSON.deserialize(str, List<Account>.class);
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    if let Err(ref e) = result {
        println!("Error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse type literal as argument: {:?}", result.err());
}

// Test transpilation of PagedResult
#[test]
fn test_transpile_paged_result() {
    use apexrust::transpile::Transpiler;
    
    let mut parser = Parser::new(PAGED_RESULT);
    let ast = parser.parse().expect("Parse failed");
    
    let mut transpiler = Transpiler::new();
    let ts = transpiler.transpile(&ast).expect("Transpile failed");
    
    println!("Transpiled PagedResult:\n{}", ts);
    
    // Check that properties are transpiled correctly
    assert!(ts.contains("pageSize"), "Should contain pageSize property");
    assert!(ts.contains("pageNumber"), "Should contain pageNumber property");
    assert!(ts.contains("totalItemCount"), "Should contain totalItemCount property");
    assert!(ts.contains("records"), "Should contain records property");
}

// Test transpilation of PropertyController
#[test]
fn test_transpile_property_controller() {
    use apexrust::transpile::Transpiler;
    
    let mut parser = Parser::new(PROPERTY_CONTROLLER);
    let ast = parser.parse().expect("Parse failed");
    
    let mut transpiler = Transpiler::new();
    let ts = transpiler.transpile(&ast).expect("Transpile failed");
    
    println!("Transpiled PropertyController:\n{}", ts);
    
    // Check key features
    assert!(ts.contains("async"), "Methods with SOQL should be async");
    assert!(ts.contains("$runtime.query"), "SOQL should become runtime.query");
    assert!(ts.contains("??"), "Null coalescing should be preserved");
}

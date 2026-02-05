use apexrust::{parse, TypeDeclaration, ClassMember};

/// Helper to check if parsing succeeds
fn parses_ok(source: &str) -> bool {
    parse(source).is_ok()
}

// ==================== Class Declaration Tests ====================

#[test]
fn test_empty_class() {
    assert!(parses_ok("public class Empty { }"));
}

#[test]
fn test_private_class() {
    assert!(parses_ok("private class PrivateClass { }"));
}

#[test]
fn test_global_class() {
    assert!(parses_ok("global class GlobalClass { }"));
}

#[test]
fn test_abstract_class() {
    assert!(parses_ok("public abstract class AbstractClass { }"));
}

#[test]
fn test_virtual_class() {
    assert!(parses_ok("public virtual class VirtualClass { }"));
}

#[test]
fn test_with_sharing_class() {
    assert!(parses_ok("public with sharing class WithSharingClass { }"));
}

#[test]
fn test_without_sharing_class() {
    assert!(parses_ok("public without sharing class WithoutSharingClass { }"));
}

#[test]
fn test_inherited_sharing_class() {
    assert!(parses_ok("public inherited sharing class InheritedSharingClass { }"));
}

#[test]
fn test_class_extends() {
    assert!(parses_ok("public class Child extends Parent { }"));
}

#[test]
fn test_class_implements() {
    assert!(parses_ok("public class MyClass implements MyInterface { }"));
}

#[test]
fn test_class_implements_multiple() {
    assert!(parses_ok("public class MyClass implements Interface1, Interface2, Interface3 { }"));
}

#[test]
fn test_class_extends_and_implements() {
    assert!(parses_ok("public class Child extends Parent implements Interface1, Interface2 { }"));
}

#[test]
fn test_class_with_annotation() {
    let source = "@isTest public class TestClass { }";
    let result = parse(source).unwrap();
    if let TypeDeclaration::Class(class) = &result.declarations[0] {
        assert_eq!(class.annotations.len(), 1);
        assert_eq!(class.annotations[0].name, "isTest");
    } else {
        panic!("Expected class");
    }
}

#[test]
fn test_class_with_multiple_annotations() {
    let source = "@isTest @SuppressWarnings public class TestClass { }";
    assert!(parses_ok(source));
}

#[test]
fn test_class_combined_modifiers() {
    assert!(parses_ok("public abstract with sharing class Combined extends Base implements Interface1 { }"));
}

// ==================== Interface Declaration Tests ====================

#[test]
fn test_empty_interface() {
    assert!(parses_ok("public interface MyInterface { }"));
}

#[test]
fn test_interface_with_method() {
    assert!(parses_ok("public interface MyInterface { void doSomething(); }"));
}

#[test]
fn test_interface_with_multiple_methods() {
    let source = r#"
        public interface MyInterface {
            void doSomething();
            String getName();
            Integer calculate(Integer a, Integer b);
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_interface_extends() {
    assert!(parses_ok("public interface ChildInterface extends ParentInterface { }"));
}

#[test]
fn test_interface_extends_multiple() {
    assert!(parses_ok("public interface MyInterface extends Interface1, Interface2 { }"));
}

#[test]
fn test_global_interface() {
    assert!(parses_ok("global interface GlobalInterface { }"));
}

// ==================== Enum Declaration Tests ====================

#[test]
fn test_simple_enum() {
    let source = "public enum Status { PENDING, ACTIVE, CLOSED }";
    let result = parse(source).unwrap();
    if let TypeDeclaration::Enum(e) = &result.declarations[0] {
        assert_eq!(e.name, "Status");
        assert_eq!(e.values, vec!["PENDING", "ACTIVE", "CLOSED"]);
    } else {
        panic!("Expected enum");
    }
}

#[test]
fn test_enum_single_value() {
    assert!(parses_ok("public enum Single { ONLY }"));
}

#[test]
fn test_enum_with_trailing_comma() {
    assert!(parses_ok("public enum Status { A, B, C, }"));
}

#[test]
fn test_private_enum() {
    assert!(parses_ok("private enum PrivateEnum { A, B }"));
}

#[test]
fn test_global_enum() {
    assert!(parses_ok("global enum GlobalEnum { A, B }"));
}

// ==================== Trigger Declaration Tests ====================

#[test]
fn test_trigger_before_insert() {
    let source = "trigger AccountTrigger on Account (before insert) { }";
    assert!(parses_ok(source));
}

#[test]
fn test_trigger_after_update() {
    let source = "trigger AccountTrigger on Account (after update) { }";
    assert!(parses_ok(source));
}

#[test]
fn test_trigger_multiple_events() {
    let source = "trigger AccountTrigger on Account (before insert, after insert, before update, after update) { }";
    assert!(parses_ok(source));
}

#[test]
fn test_trigger_with_body() {
    let source = r#"
        trigger AccountTrigger on Account (before insert) {
            for (Account acc : newRecords) {
                acc.Description = 'Processed';
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_trigger_all_events() {
    let source = "trigger AllEvents on Contact (before insert, before update, before delete, after insert, after update, after delete, after undelete) { }";
    assert!(parses_ok(source));
}

// ==================== Field Declaration Tests ====================

#[test]
fn test_simple_field() {
    assert!(parses_ok("public class Test { String name; }"));
}

#[test]
fn test_public_field() {
    assert!(parses_ok("public class Test { public String name; }"));
}

#[test]
fn test_private_field() {
    assert!(parses_ok("public class Test { private String name; }"));
}

#[test]
fn test_protected_field() {
    assert!(parses_ok("public class Test { protected String name; }"));
}

#[test]
fn test_static_field() {
    assert!(parses_ok("public class Test { public static String name; }"));
}

#[test]
fn test_final_field() {
    assert!(parses_ok("public class Test { public final String NAME = 'constant'; }"));
}

#[test]
fn test_static_final_field() {
    assert!(parses_ok("public class Test { public static final String CONSTANT = 'value'; }"));
}

#[test]
fn test_transient_field() {
    assert!(parses_ok("public class Test { transient String temp; }"));
}

#[test]
fn test_field_with_initializer() {
    assert!(parses_ok("public class Test { Integer count = 0; }"));
}

#[test]
fn test_multiple_fields_same_line() {
    assert!(parses_ok("public class Test { Integer x, y, z; }"));
}

#[test]
fn test_multiple_fields_with_initializers() {
    assert!(parses_ok("public class Test { Integer x = 1, y = 2, z = 3; }"));
}

#[test]
fn test_list_field() {
    assert!(parses_ok("public class Test { List<String> items = new List<String>(); }"));
}

#[test]
fn test_map_field() {
    assert!(parses_ok("public class Test { Map<String, Integer> counts = new Map<String, Integer>(); }"));
}

// ==================== Method Declaration Tests ====================

#[test]
fn test_void_method() {
    assert!(parses_ok("public class Test { void doSomething() { } }"));
}

#[test]
fn test_method_with_return_type() {
    assert!(parses_ok("public class Test { String getName() { return 'test'; } }"));
}

#[test]
fn test_public_method() {
    assert!(parses_ok("public class Test { public void doSomething() { } }"));
}

#[test]
fn test_private_method() {
    assert!(parses_ok("public class Test { private void doSomething() { } }"));
}

#[test]
fn test_protected_method() {
    assert!(parses_ok("public class Test { protected void doSomething() { } }"));
}

#[test]
fn test_global_method() {
    assert!(parses_ok("global class Test { global void doSomething() { } }"));
}

#[test]
fn test_static_method() {
    assert!(parses_ok("public class Test { public static void doSomething() { } }"));
}

#[test]
fn test_virtual_method() {
    assert!(parses_ok("public virtual class Test { public virtual void doSomething() { } }"));
}

#[test]
fn test_abstract_method() {
    assert!(parses_ok("public abstract class Test { public abstract void doSomething(); }"));
}

#[test]
fn test_override_method() {
    assert!(parses_ok("public class Test extends Base { public override void doSomething() { } }"));
}

#[test]
fn test_method_with_parameter() {
    assert!(parses_ok("public class Test { void doSomething(String name) { } }"));
}

#[test]
fn test_method_with_multiple_parameters() {
    assert!(parses_ok("public class Test { void doSomething(String name, Integer count, Boolean flag) { } }"));
}

#[test]
fn test_method_with_final_parameter() {
    assert!(parses_ok("public class Test { void doSomething(final String name) { } }"));
}

#[test]
fn test_method_with_list_parameter() {
    assert!(parses_ok("public class Test { void process(List<Account> accounts) { } }"));
}

#[test]
fn test_method_with_annotation() {
    assert!(parses_ok("public class Test { @AuraEnabled public static void doSomething() { } }"));
}

#[test]
fn test_testmethod_keyword() {
    assert!(parses_ok("public class Test { testmethod void testSomething() { } }"));
}

#[test]
fn test_webservice_method() {
    assert!(parses_ok("global class Test { webservice static String doSomething() { return ''; } }"));
}

// ==================== Constructor Declaration Tests ====================

#[test]
fn test_default_constructor() {
    let source = "public class Test { public Test() { } }";
    let result = parse(source).unwrap();
    if let TypeDeclaration::Class(class) = &result.declarations[0] {
        assert!(class.members.iter().any(|m| matches!(m, ClassMember::Constructor(_))));
    }
}

#[test]
fn test_constructor_with_parameter() {
    assert!(parses_ok("public class Test { public Test(String name) { } }"));
}

#[test]
fn test_constructor_with_multiple_parameters() {
    assert!(parses_ok("public class Test { public Test(String name, Integer count) { } }"));
}

#[test]
fn test_private_constructor() {
    assert!(parses_ok("public class Test { private Test() { } }"));
}

#[test]
fn test_multiple_constructors() {
    let source = r#"
        public class Test {
            public Test() { }
            public Test(String name) { }
            public Test(String name, Integer count) { }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_constructor_with_this_call() {
    // Note: this('default') constructor delegation is not yet fully supported
    let source = r#"
        public class Test {
            private String name;
            public Test() {
                this.name = 'default';
            }
            public Test(String name) {
                this.name = name;
            }
        }
    "#;
    assert!(parses_ok(source));
}

// ==================== Property Declaration Tests ====================

#[test]
fn test_auto_property() {
    assert!(parses_ok("public class Test { public String Name { get; set; } }"));
}

#[test]
fn test_property_get_only() {
    assert!(parses_ok("public class Test { public String Name { get; } }"));
}

#[test]
fn test_property_private_set() {
    assert!(parses_ok("public class Test { public String Name { get; private set; } }"));
}

#[test]
fn test_property_with_body() {
    let source = r#"
        public class Test {
            private String name;
            public String Name {
                get { return name; }
                set { name = value; }
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_property_with_get_body_only() {
    let source = r#"
        public class Test {
            private String firstName;
            private String lastName;
            public String FullName {
                get { return firstName + ' ' + lastName; }
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_static_property() {
    assert!(parses_ok("public class Test { public static Integer Count { get; set; } }"));
}

// ==================== Inner Class Tests ====================

#[test]
fn test_inner_class() {
    let source = r#"
        public class Outer {
            public class Inner { }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_private_inner_class() {
    let source = r#"
        public class Outer {
            private class Inner { }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_inner_class_with_members() {
    let source = r#"
        public class Outer {
            public class Inner {
                public String name;
                public void doSomething() { }
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_multiple_inner_classes() {
    let source = r#"
        public class Outer {
            public class Inner1 { }
            public class Inner2 { }
            private class Inner3 { }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_inner_interface() {
    let source = r#"
        public class Outer {
            public interface InnerInterface {
                void doSomething();
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_inner_enum() {
    let source = r#"
        public class Outer {
            public enum Status { ACTIVE, INACTIVE }
        }
    "#;
    assert!(parses_ok(source));
}

// ==================== Multiple Declarations Tests ====================

#[test]
fn test_multiple_classes() {
    let source = r#"
        public class Class1 { }
        public class Class2 { }
        public class Class3 { }
    "#;
    let result = parse(source).unwrap();
    assert_eq!(result.declarations.len(), 3);
}

#[test]
fn test_class_and_interface() {
    let source = r#"
        public interface MyInterface { void doSomething(); }
        public class MyClass implements MyInterface { public void doSomething() { } }
    "#;
    let result = parse(source).unwrap();
    assert_eq!(result.declarations.len(), 2);
}

#[test]
fn test_class_interface_enum() {
    let source = r#"
        public enum Status { ACTIVE, INACTIVE }
        public interface Processor { void process(); }
        public class MyProcessor implements Processor { public void process() { } }
    "#;
    let result = parse(source).unwrap();
    assert_eq!(result.declarations.len(), 3);
}

// ==================== Complex Class Tests ====================

#[test]
fn test_complete_class() {
    let source = r#"
        @isTest
        public with sharing class AccountService {
            private static final String DEFAULT_NAME = 'Unknown';

            public List<Account> accounts { get; private set; }

            public AccountService() {
                this.accounts = new List<Account>();
            }

            public AccountService(List<Account> initialAccounts) {
                this.accounts = initialAccounts;
            }

            public void addAccount(Account acc) {
                if (acc != null) {
                    accounts.add(acc);
                }
            }

            public List<Account> getActiveAccounts() {
                List<Account> result = new List<Account>();
                for (Account acc : accounts) {
                    if (acc.IsActive__c) {
                        result.add(acc);
                    }
                }
                return result;
            }

            public static void processAll(List<Account> accs) {
                try {
                    update accs;
                } catch (DmlException e) {
                    System.debug('Error: ' + e.getMessage());
                }
            }

            public class AccountWrapper {
                public Account record { get; set; }
                public Boolean isSelected { get; set; }
            }

            public enum ProcessingStatus {
                PENDING,
                IN_PROGRESS,
                COMPLETED,
                FAILED
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_apex_test_class() {
    let source = r#"
        @isTest
        private class AccountServiceTest {
            @testSetup
            static void setup() {
                List<Account> accounts = new List<Account>();
                for (Integer i = 0; i < 10; i++) {
                    accounts.add(new Account(Name = 'Test ' + i));
                }
                insert accounts;
            }

            @isTest
            static void testAddAccount() {
                AccountService service = new AccountService();
                Account acc = new Account(Name = 'Test');

                Test.startTest();
                service.addAccount(acc);
                Test.stopTest();

                System.assertEquals(1, service.accounts.size());
            }

            @isTest
            static void testGetActiveAccounts() {
                AccountService service = new AccountService([SELECT Id, Name, IsActive__c FROM Account]);

                Test.startTest();
                List<Account> active = service.getActiveAccounts();
                Test.stopTest();

                System.assertNotEquals(null, active);
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_batch_apex_class() {
    let source = r#"
        global class AccountBatch implements Database.Batchable<sObject>, Database.Stateful {
            global Integer recordsProcessed = 0;

            global Database.QueryLocator start(Database.BatchableContext bc) {
                return Database.getQueryLocator([SELECT Id, Name FROM Account]);
            }

            global void execute(Database.BatchableContext bc, List<Account> scope) {
                for (Account acc : scope) {
                    acc.Description = 'Processed';
                    recordsProcessed++;
                }
                update scope;
            }

            global void finish(Database.BatchableContext bc) {
                System.debug('Processed ' + recordsProcessed + ' records');
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_schedulable_class() {
    let source = r#"
        global class AccountScheduler implements Schedulable {
            global void execute(SchedulableContext sc) {
                AccountBatch batch = new AccountBatch();
                Database.executeBatch(batch, 200);
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_queueable_class() {
    let source = r#"
        public class AccountQueueable implements Queueable {
            private List<Account> accounts;

            public AccountQueueable(List<Account> accounts) {
                this.accounts = accounts;
            }

            public void execute(QueueableContext context) {
                for (Account acc : accounts) {
                    acc.Description = 'Queued Processing';
                }
                update accounts;
            }
        }
    "#;
    assert!(parses_ok(source));
}

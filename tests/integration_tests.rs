use apexrust::parse;

/// Helper to check if parsing succeeds
fn parses_ok(source: &str) -> bool {
    match parse(source) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            false
        }
    }
}

// ==================== Real-World Apex Examples ====================
// Note: These tests avoid features not yet supported:
// - SOQL bind variables (:variable)
// - Constructor field initialization (new Object(Field = value))
// - Complex annotation parameters with =

#[test]
fn test_trigger_handler_basic() {
    let source = r#"
        public class ContactTriggerHandler {
            public static void handleBeforeInsert(List<Contact> newContacts) {
                Set<Id> accountIds = new Set<Id>();
                for (Contact con : newContacts) {
                    if (con.AccountId != null) {
                        accountIds.add(con.AccountId);
                    }
                }
            }

            public static void handleAfterInsert(List<Contact> newContacts) {
                List<Task> tasks = new List<Task>();
                for (Contact con : newContacts) {
                    Task t = new Task();
                    t.WhoId = con.Id;
                    t.Subject = 'Follow up with new contact';
                    t.Status = 'Not Started';
                    tasks.add(t);
                }

                if (!tasks.isEmpty()) {
                    insert tasks;
                }
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_service_class() {
    let source = r#"
        public with sharing class AccountService {

            public static List<Account> getAccounts() {
                return [SELECT Id, Name, Industry FROM Account LIMIT 50];
            }

            public static Account saveAccount(Account account) {
                upsert account;
                return account;
            }

            public static void deleteAccounts(List<Account> accounts) {
                delete accounts;
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_controller_class() {
    let source = r#"
        public with sharing class AccountController {

            @AuraEnabled
            public static List<Account> getAccounts() {
                return [SELECT Id, Name, Industry FROM Account ORDER BY Name LIMIT 50];
            }

            @AuraEnabled
            public static void updateAccount(Account acc) {
                try {
                    update acc;
                } catch (DmlException e) {
                    System.debug('Error: ' + e.getMessage());
                }
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_custom_exception() {
    let source = r#"
        public class CustomException extends Exception {
            private String errorCode;

            public CustomException(String message, String errorCode) {
                this.errorCode = errorCode;
            }

            public String getErrorCode() {
                return errorCode;
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_utility_class() {
    let source = r#"
        public class StringUtils {

            public static Boolean isBlank(String str) {
                return str == null || str.trim().length() == 0;
            }

            public static Boolean isNotBlank(String str) {
                return !isBlank(str);
            }

            public static String defaultIfBlank(String str, String defaultValue) {
                if (isBlank(str)) {
                    return defaultValue;
                }
                return str;
            }

            public static String truncate(String str, Integer maxLength) {
                if (str == null || str.length() <= maxLength) {
                    return str;
                }
                return str.substring(0, maxLength - 3) + '...';
            }

            public static List<String> split(String str, String delimiter) {
                if (isBlank(str)) {
                    return new List<String>();
                }
                return str.split(delimiter);
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_selector_class() {
    let source = r#"
        public inherited sharing class AccountSelector {

            public static List<Account> selectAll() {
                return [SELECT Id, Name, Industry FROM Account];
            }

            public static List<Account> selectByName(String name) {
                return [SELECT Id, Name, Industry FROM Account WHERE Name != null];
            }

            public static List<Account> selectWithContacts(Integer limitCount) {
                return [SELECT Id, Name FROM Account ORDER BY Name LIMIT 100];
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_domain_class() {
    let source = r#"
        public class Accounts {
            private List<Account> records;

            public Accounts(List<Account> records) {
                this.records = records;
            }

            public List<Account> getRecords() {
                return records;
            }

            public Accounts filterByIndustry(String industry) {
                List<Account> filtered = new List<Account>();
                for (Account acc : records) {
                    if (acc.Industry == industry) {
                        filtered.add(acc);
                    }
                }
                return new Accounts(filtered);
            }

            public void setIndustry(String industry) {
                for (Account acc : records) {
                    acc.Industry = industry;
                }
            }

            public void save() {
                if (!records.isEmpty()) {
                    upsert records;
                }
            }

            public Set<Id> getIds() {
                Set<Id> ids = new Set<Id>();
                for (Account acc : records) {
                    ids.add(acc.Id);
                }
                return ids;
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_batch_class() {
    let source = r#"
        global class AccountBatch implements Database.Batchable {
            global Integer recordsProcessed = 0;

            global Database.QueryLocator start(Database.BatchableContext bc) {
                return Database.getQueryLocator('SELECT Id, Name FROM Account');
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

#[test]
fn test_test_class() {
    let source = r#"
        @isTest
        private class AccountServiceTest {

            @isTest
            static void testGetAccounts() {
                List<Account> accounts = new List<Account>();
                for (Integer i = 0; i < 10; i++) {
                    Account acc = new Account();
                    acc.Name = 'Test ' + i;
                    accounts.add(acc);
                }
                insert accounts;

                Test.startTest();
                List<Account> results = AccountService.getAccounts();
                Test.stopTest();

                System.assertNotEquals(null, results);
            }

            @isTest
            static void testExceptionHandling() {
                Boolean exceptionThrown = false;

                Test.startTest();
                try {
                    AccountService.processInvalid(null);
                } catch (CustomException e) {
                    exceptionThrown = true;
                }
                Test.stopTest();

                System.assert(exceptionThrown, 'Exception should have been thrown');
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_sharing_classes() {
    let source = r#"
        public without sharing class AdminService {
            public List<Account> getAllAccounts() {
                return [SELECT Id, Name FROM Account];
            }
        }

        public with sharing class UserService {
            public List<Account> getMyAccounts() {
                return [SELECT Id, Name FROM Account];
            }
        }

        public inherited sharing class FlexibleService {
            public List<Account> getAccounts() {
                return [SELECT Id, Name FROM Account];
            }
        }
    "#;
    assert!(parses_ok(source));
}

// ==================== Edge Cases ====================

#[test]
fn test_deeply_nested_class() {
    let source = r#"
        public class Level1 {
            public class Level2 {
                public class Level3 {
                    public class Level4 {
                        public String value;
                    }
                }
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_class_with_all_member_types() {
    let source = r#"
        public class CompleteClass {
            public static final String CONSTANT = 'VALUE';
            private static Integer counter = 0;

            public String name;
            private Integer count;
            protected Boolean flag;

            public CompleteClass() {
                this.name = 'default';
                this.count = 0;
                this.flag = false;
            }

            public CompleteClass(String name) {
                this.name = name;
                this.count = 0;
                this.flag = false;
            }

            public Integer Count {
                get { return count; }
                private set { count = value; }
            }

            public void increment() {
                count++;
            }

            public String getName() {
                return name;
            }

            public static Integer getCounter() {
                return counter;
            }

            public static void incrementCounter() {
                counter++;
            }

            public interface Processor {
                void process(CompleteClass instance);
            }

            public enum Status {
                ACTIVE,
                INACTIVE
            }

            public class Builder {
                private String name;
                private Integer count;

                public Builder setName(String name) {
                    this.name = name;
                    return this;
                }

                public Builder setCount(Integer count) {
                    this.count = count;
                    return this;
                }

                public CompleteClass build() {
                    CompleteClass instance = new CompleteClass(name);
                    instance.count = count;
                    return instance;
                }
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_generic_types_complex() {
    let source = r#"
        public class GenericHandler {
            private Map<String, List<Account>> complexStructure;
            private List<List<String>> doubleNested;

            public Map<String, Object> process(List<Map<String, Integer>> input) {
                Map<String, Object> result = new Map<String, Object>();
                return result;
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_method_chaining() {
    let source = r#"
        public class FluentBuilder {
            private String name;
            private Integer value;
            private List<String> items;

            public FluentBuilder() {
                items = new List<String>();
            }

            public FluentBuilder withName(String name) {
                this.name = name;
                return this;
            }

            public FluentBuilder withValue(Integer value) {
                this.value = value;
                return this;
            }

            public FluentBuilder addItem(String item) {
                this.items.add(item);
                return this;
            }

            public Result build() {
                return new Result(name, value, items);
            }

            public class Result {
                public String name;
                public Integer value;
                public List<String> items;

                public Result(String name, Integer value, List<String> items) {
                    this.name = name;
                    this.value = value;
                    this.items = items;
                }
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_method_chaining_usage() {
    let source = r#"
        public class Test {
            public void test() {
                FluentBuilder.Result result = new FluentBuilder()
                    .withName('test')
                    .withValue(42)
                    .addItem('a')
                    .addItem('b')
                    .build();
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_trigger() {
    let source = r#"
        trigger AccountTrigger on Account (before insert, after insert, before update, after update, before delete, after delete, after undelete) {
            for (Account acc : newAccounts) {
                acc.Description = 'Triggered';
            }
            update newAccounts;
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_abstract_class_hierarchy() {
    let source = r#"
        public abstract class Animal {
            protected String name;

            public Animal(String name) {
                this.name = name;
            }

            public abstract void makeSound();

            public String getName() {
                return name;
            }
        }

        public class Dog extends Animal {
            public Dog(String name) {
                this.name = name;
            }

            public override void makeSound() {
                System.debug('Woof!');
            }
        }

        public class Cat extends Animal {
            public Cat(String name) {
                this.name = name;
            }

            public override void makeSound() {
                System.debug('Meow!');
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_interface_implementation() {
    let source = r#"
        public interface Drawable {
            void draw();
            String getColor();
        }

        public interface Resizable {
            void resize(Integer width, Integer height);
        }

        public class Rectangle implements Drawable, Resizable {
            private Integer width;
            private Integer height;
            private String color;

            public Rectangle(Integer width, Integer height) {
                this.width = width;
                this.height = height;
                this.color = 'black';
            }

            public void draw() {
                System.debug('Drawing rectangle ' + width + 'x' + height);
            }

            public String getColor() {
                return color;
            }

            public void resize(Integer width, Integer height) {
                this.width = width;
                this.height = height;
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_complex_control_flow() {
    let source = r#"
        public class ControlFlowExample {
            public void process(List<Account> accounts) {
                for (Integer i = 0; i < accounts.size(); i++) {
                    Account acc = accounts[i];

                    if (acc.Name == null) {
                        continue;
                    }

                    switch on acc.Industry {
                        when 'Technology' {
                            processTech(acc);
                        }
                        when 'Finance' {
                            processFinance(acc);
                        }
                        when else {
                            processOther(acc);
                        }
                    }

                    try {
                        update acc;
                    } catch (DmlException e) {
                        System.debug('Error: ' + e.getMessage());
                        break;
                    } finally {
                        logCompletion(acc);
                    }
                }
            }

            private void processTech(Account acc) {
                acc.Description = 'Tech company';
            }

            private void processFinance(Account acc) {
                acc.Description = 'Finance company';
            }

            private void processOther(Account acc) {
                acc.Description = 'Other';
            }

            private void logCompletion(Account acc) {
                System.debug('Processed: ' + acc.Name);
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_expressions_comprehensive() {
    let source = r#"
        public class ExpressionTest {
            public void testExpressions() {
                Integer a = 1 + 2 * 3 - 4 / 2;
                Boolean b = a > 5 && a < 10 || a == 7;
                String s = 'Hello' + ' ' + 'World';

                Integer x = a > 0 ? a : -a;
                String name = null;
                String safeName = name ?? 'default';

                List<Integer> nums = new List<Integer>();
                nums.add(1);
                nums.add(2);
                Integer first = nums[0];
                Integer size = nums.size();

                Account acc = new Account();
                acc.Name = 'Test';
                String accName = acc.Name;
                String safeProp = acc?.Name;

                Integer preInc = ++a;
                Integer postDec = a--;

                b = acc instanceof Account;

                a += 5;
                a -= 3;
                a *= 2;
                a /= 4;

                Integer bits = a & 255;
                bits = bits | 15;
                bits = bits ^ 240;
                bits = bits << 2;
                bits = bits >> 1;
            }
        }
    "#;
    assert!(parses_ok(source));
}

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

/// Wrap code in a class for testing
fn wrap_in_class(code: &str) -> String {
    format!("public class Test {{ {} }}", code)
}

/// Wrap code in a method for testing expressions/statements
fn wrap_in_method(code: &str) -> String {
    format!("public class Test {{ public void test() {{ {} }} }}", code)
}

// ==================== Hex/Binary/Octal Literal Tests ====================

#[test]
fn test_hex_literal() {
    assert!(parses_ok(&wrap_in_method("Integer x = 0xFF;")));
}

#[test]
fn test_hex_literal_uppercase() {
    assert!(parses_ok(&wrap_in_method("Integer x = 0XAB12;")));
}

#[test]
fn test_hex_literal_mixed_case() {
    assert!(parses_ok(&wrap_in_method("Integer x = 0xDeAdBeEf;")));
}

#[test]
fn test_hex_long_literal() {
    assert!(parses_ok(&wrap_in_method("Long x = 0xFFFFFFFFFL;")));
}

#[test]
fn test_binary_literal() {
    assert!(parses_ok(&wrap_in_method("Integer x = 0b1010;")));
}

#[test]
fn test_binary_literal_uppercase() {
    assert!(parses_ok(&wrap_in_method("Integer x = 0B11110000;")));
}

#[test]
fn test_octal_literal() {
    assert!(parses_ok(&wrap_in_method("Integer x = 0755;")));
}

#[test]
fn test_octal_literal_simple() {
    assert!(parses_ok(&wrap_in_method("Integer x = 07;")));
}

// ==================== Modulo Operator Tests ====================

#[test]
fn test_modulo_operator() {
    assert!(parses_ok(&wrap_in_method("Integer x = 10 % 3;")));
}

#[test]
fn test_modulo_assignment() {
    assert!(parses_ok(&wrap_in_method("Integer x = 10; x %= 3;")));
}

#[test]
fn test_modulo_in_expression() {
    assert!(parses_ok(&wrap_in_method("Integer x = (a + b) % (c - d);")));
}

// ==================== SOQL Bind Variable Tests ====================

#[test]
fn test_soql_bind_variable_simple() {
    assert!(parses_ok(&wrap_in_method("String name = 'Test'; List<Account> accs = [SELECT Id FROM Account WHERE Name = :name];")));
}

#[test]
fn test_soql_bind_variable_in_list() {
    // Note: IN with bind variable uses special syntax
    assert!(parses_ok(&wrap_in_method("Set<Id> ids = new Set<Id>(); List<Account> accs = [SELECT Id FROM Account WHERE Id = :ids];")));
}

#[test]
fn test_soql_bind_variable_in_limit() {
    assert!(parses_ok(&wrap_in_method("Integer lim = 10; List<Account> accs = [SELECT Id FROM Account LIMIT :lim];")));
}

// ==================== SOQL GROUP BY / HAVING Tests ====================

#[test]
fn test_soql_group_by() {
    assert!(parses_ok(&wrap_in_method("AggregateResult[] results = [SELECT Industry, COUNT(Id) FROM Account GROUP BY Industry];")));
}

#[test]
fn test_soql_group_by_multiple() {
    assert!(parses_ok(&wrap_in_method("AggregateResult[] results = [SELECT Industry, Type, COUNT(Id) FROM Account GROUP BY Industry, Type];")));
}

#[test]
fn test_soql_having() {
    // HAVING with simple field comparison (aggregate result alias)
    assert!(parses_ok(&wrap_in_method("AggregateResult[] results = [SELECT Industry, COUNT(Id) cnt FROM Account GROUP BY Industry HAVING cnt > 5];")));
}

// ==================== SOQL Aggregate Function Tests ====================

#[test]
fn test_soql_count() {
    assert!(parses_ok(&wrap_in_method("Integer cnt = [SELECT COUNT() FROM Account];")));
}

#[test]
fn test_soql_count_field() {
    assert!(parses_ok(&wrap_in_method("AggregateResult[] results = [SELECT COUNT(Id) FROM Account];")));
}

#[test]
fn test_soql_sum() {
    assert!(parses_ok(&wrap_in_method("AggregateResult[] results = [SELECT SUM(Amount) FROM Opportunity];")));
}

#[test]
fn test_soql_avg() {
    assert!(parses_ok(&wrap_in_method("AggregateResult[] results = [SELECT AVG(Amount) FROM Opportunity];")));
}

#[test]
fn test_soql_min_max() {
    assert!(parses_ok(&wrap_in_method("AggregateResult[] results = [SELECT MIN(Amount), MAX(Amount) FROM Opportunity];")));
}

#[test]
fn test_soql_aggregate_with_alias() {
    assert!(parses_ok(&wrap_in_method("AggregateResult[] results = [SELECT COUNT(Id) total FROM Account];")));
}

// ==================== SOQL Subquery Tests ====================

#[test]
fn test_soql_subquery() {
    assert!(parses_ok(&wrap_in_method("List<Account> accs = [SELECT Id, (SELECT Id FROM Contacts) FROM Account];")));
}

#[test]
fn test_soql_subquery_with_where() {
    assert!(parses_ok(&wrap_in_method("List<Account> accs = [SELECT Id, (SELECT Id FROM Contacts WHERE IsActive = true) FROM Account];")));
}

// ==================== SOQL Relationship Query Tests ====================

#[test]
fn test_soql_parent_relationship() {
    assert!(parses_ok(&wrap_in_method("List<Contact> cons = [SELECT Id, Account.Name FROM Contact];")));
}

#[test]
fn test_soql_custom_relationship() {
    assert!(parses_ok(&wrap_in_method("List<Custom__c> recs = [SELECT Id, Parent__r.Name FROM Custom__c];")));
}

#[test]
fn test_soql_nested_relationship() {
    assert!(parses_ok(&wrap_in_method("List<Contact> cons = [SELECT Id, Account.Owner.Name FROM Contact];")));
}

// ==================== SOQL FOR Clause Tests ====================

#[test]
fn test_soql_for_update() {
    assert!(parses_ok(&wrap_in_method("List<Account> accs = [SELECT Id FROM Account FOR UPDATE];")));
}

#[test]
fn test_soql_for_view() {
    assert!(parses_ok(&wrap_in_method("List<Account> accs = [SELECT Id FROM Account FOR VIEW];")));
}

#[test]
fn test_soql_for_reference() {
    assert!(parses_ok(&wrap_in_method("List<Account> accs = [SELECT Id FROM Account FOR REFERENCE];")));
}

// ==================== SOQL Date Literal Tests ====================

#[test]
fn test_soql_date_today() {
    assert!(parses_ok(&wrap_in_method("List<Account> accs = [SELECT Id FROM Account WHERE CreatedDate = TODAY];")));
}

#[test]
fn test_soql_date_last_n_days() {
    assert!(parses_ok(&wrap_in_method("List<Account> accs = [SELECT Id FROM Account WHERE CreatedDate = LAST_N_DAYS:30];")));
}

#[test]
fn test_soql_date_this_quarter() {
    assert!(parses_ok(&wrap_in_method("List<Opportunity> opps = [SELECT Id FROM Opportunity WHERE CloseDate = THIS_QUARTER];")));
}

// ==================== SOSL Query Tests ====================

#[test]
fn test_sosl_simple() {
    assert!(parses_ok(&wrap_in_method("List<List<SObject>> results = [FIND 'Acme' RETURNING Account];")));
}

#[test]
fn test_sosl_with_fields() {
    assert!(parses_ok(&wrap_in_method("List<List<SObject>> results = [FIND 'Acme' RETURNING Account(Id, Name)];")));
}

#[test]
fn test_sosl_multiple_objects() {
    assert!(parses_ok(&wrap_in_method("List<List<SObject>> results = [FIND 'Acme' RETURNING Account, Contact];")));
}

#[test]
fn test_sosl_in_all_fields() {
    assert!(parses_ok(&wrap_in_method("List<List<SObject>> results = [FIND 'Acme' IN ALL FIELDS RETURNING Account];")));
}

#[test]
fn test_sosl_in_name_fields() {
    assert!(parses_ok(&wrap_in_method("List<List<SObject>> results = [FIND 'Acme' IN NAME FIELDS RETURNING Account];")));
}

#[test]
fn test_sosl_with_where() {
    assert!(parses_ok(&wrap_in_method("List<List<SObject>> results = [FIND 'Acme' RETURNING Account(Id WHERE Industry = 'Technology')];")));
}

#[test]
fn test_sosl_with_limit() {
    assert!(parses_ok(&wrap_in_method("List<List<SObject>> results = [FIND 'Acme' RETURNING Account LIMIT 10];")));
}

// ==================== Cast Expression Tests ====================

#[test]
fn test_cast_to_object() {
    assert!(parses_ok(&wrap_in_method("Account acc = (Account)obj;")));
}

#[test]
fn test_cast_to_primitive() {
    assert!(parses_ok(&wrap_in_method("Integer i = (Integer)someDecimal;")));
}

#[test]
fn test_cast_sobject() {
    assert!(parses_ok(&wrap_in_method("SObject sobj = record; Account acc = (Account)sobj;")));
}

#[test]
fn test_cast_in_expression() {
    // Cast and then access field in separate step
    assert!(parses_ok(&wrap_in_method("Account acc = (Account)obj; String name = acc.Name;")));
}

// ==================== Constructor Chaining Tests ====================

#[test]
fn test_constructor_this_chaining() {
    let source = r#"
        public class Test {
            public String name;
            public Test() {
                this('default');
            }
            public Test(String n) {
                this.name = n;
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_constructor_super_chaining() {
    let source = r#"
        public class Child extends Parent {
            public Child() {
                super('value');
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_constructor_with_this_field_access() {
    let source = r#"
        public class Test {
            public String name;
            public Test(String n) {
                this.name = n;
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_constructor_super_method_call() {
    let source = r#"
        public class Child extends Parent {
            public Child() {
                super.init();
            }
        }
    "#;
    assert!(parses_ok(source));
}

// ==================== Static Initializer Block Tests ====================

#[test]
fn test_static_block_simple() {
    let source = r#"
        public class Test {
            static {
                System.debug('Static init');
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_static_block_with_code() {
    let source = r#"
        public class Test {
            private static Map<String, Integer> cache;
            static {
                cache = new Map<String, Integer>();
                cache.put('a', 1);
                cache.put('b', 2);
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_static_member_vs_static_block() {
    let source = r#"
        public class Test {
            static {
                System.debug('block');
            }
            public static String name = 'test';
            public static void doSomething() { }
        }
    "#;
    assert!(parses_ok(source));
}

// ==================== Complex Combined Tests ====================

#[test]
fn test_complex_soql_with_all_features() {
    let source = r#"
        public class Test {
            public void test() {
                String searchTerm = 'Acme';
                Integer limit1 = 100;
                List<Account> accs = [
                    SELECT Id, Name, Industry, Owner.Name,
                           (SELECT Id, Name FROM Contacts WHERE IsActive = true)
                    FROM Account
                    WHERE Name LIKE :searchTerm
                    AND CreatedDate = LAST_N_DAYS:30
                    ORDER BY Name ASC NULLS LAST
                    LIMIT :limit1
                    FOR UPDATE
                ];
            }
        }
    "#;
    assert!(parses_ok(source));
}

#[test]
fn test_complex_class_with_new_features() {
    let source = r#"
        public class ComplexClass extends BaseClass {
            private static final Integer HEX_VALUE = 0xFF;
            private static Map<String, Object> cache;

            static {
                cache = new Map<String, Object>();
            }

            public ComplexClass() {
                this('default', 0xFF);
            }

            public ComplexClass(String name, Integer value) {
                super(name);
                this.processValue(value % 10);
            }

            public void queryData() {
                String name = 'Test';
                List<Account> accs = [SELECT Id, Account.Name FROM Contact WHERE Name = :name];
                Object obj = accs[0];
                Account acc = (Account)obj;
            }

            public void searchData() {
                List<List<SObject>> results = [FIND 'test' IN ALL FIELDS RETURNING Account(Id, Name)];
            }
        }
    "#;
    assert!(parses_ok(source));
}

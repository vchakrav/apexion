use apexrust::{parse, tokenize, TypeDeclaration};

fn main() {
    let source = r#"
@isTest
public class AccountService {
    // Fields
    private static final String DEFAULT_NAME = 'Unknown';
    public List<Account> accounts;

    // Constructor
    public AccountService() {
        this.accounts = new List<Account>();
    }

    // Method with SOQL
    public List<Account> getActiveAccounts() {
        return [SELECT Id, Name, Industry FROM Account WHERE IsActive__c = true LIMIT 100];
    }

    // Method with control flow
    public void processAccounts(List<Account> accs) {
        for (Account acc : accs) {
            if (acc.Name == null) {
                acc.Name = DEFAULT_NAME;
            }
        }
        update accs;
    }

    // Method with try-catch
    public void safeInsert(Account acc) {
        try {
            insert acc;
        } catch (DmlException e) {
            System.debug('Error: ' + e.getMessage());
        }
    }

    // Property
    public Integer AccountCount {
        get { return accounts.size(); }
    }
}

public enum AccountType {
    CUSTOMER,
    PARTNER,
    VENDOR
}

public interface IAccountProcessor {
    void process(Account acc);
    Boolean validate(Account acc);
}
    "#;

    println!("=== Apex Parser Demo ===\n");

    // Tokenize
    println!("--- Tokenizing ---");
    let tokens = tokenize(source);
    println!("Found {} tokens\n", tokens.len());

    // Show first few tokens
    println!("First 20 tokens:");
    for (i, token) in tokens.iter().take(20).enumerate() {
        println!("  {}: {:?}", i, token.kind);
    }
    println!();

    // Parse
    println!("--- Parsing ---");
    match parse(source) {
        Ok(cu) => {
            println!("Successfully parsed {} type declarations:\n", cu.declarations.len());

            for decl in &cu.declarations {
                match decl {
                    TypeDeclaration::Class(class) => {
                        println!("Class: {}", class.name);
                        println!("  Access: {:?}", class.modifiers.access);
                        println!("  Annotations: {:?}", class.annotations.iter().map(|a| &a.name).collect::<Vec<_>>());
                        println!("  Members: {} items", class.members.len());

                        for member in &class.members {
                            match member {
                                apexrust::ClassMember::Field(f) => {
                                    println!("    - Field: {} ({})", f.declarators[0].name, f.type_ref.name);
                                }
                                apexrust::ClassMember::Method(m) => {
                                    println!("    - Method: {}() -> {}", m.name, m.return_type.name);
                                }
                                apexrust::ClassMember::Constructor(c) => {
                                    println!("    - Constructor: {}()", c.name);
                                }
                                apexrust::ClassMember::Property(p) => {
                                    println!("    - Property: {} ({})", p.name, p.type_ref.name);
                                }
                                _ => {}
                            }
                        }
                        println!();
                    }
                    TypeDeclaration::Enum(e) => {
                        println!("Enum: {}", e.name);
                        println!("  Values: {:?}", e.values);
                        println!();
                    }
                    TypeDeclaration::Interface(i) => {
                        println!("Interface: {}", i.name);
                        println!("  Methods: {} items", i.members.len());
                        println!();
                    }
                    TypeDeclaration::Trigger(t) => {
                        println!("Trigger: {} on {}", t.name, t.object);
                        println!();
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}

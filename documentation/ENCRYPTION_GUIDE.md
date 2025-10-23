# Field-Level Encryption Guide

## Overview

This application implements **field-level encryption** using AES-256-GCM to protect sensitive string data in the database. Even if an attacker gains access to the database, they cannot read the encrypted fields without the encryption key.

## Security Architecture

### Encryption Algorithm: AES-256-GCM

**AES-256-GCM** provides:
- **Confidentiality**: Data is encrypted and unreadable without the key
- **Authenticity**: Tampering is detected via authentication tag
- **Integrity**: Modified ciphertext fails decryption
- **Performance**: Hardware-accelerated on modern CPUs

### Key Features

1. **Transparent Encryption**: Fields are automatically encrypted when saved to the database
2. **Transparent Decryption**: Fields are automatically decrypted when loaded from the database
3. **Unique Nonces**: Each encryption uses a random 12-byte nonce (prevents pattern analysis)
4. **Memory Safety**: Sensitive data is zeroized when dropped from memory
5. **Diesel ORM Integration**: Works seamlessly with your existing models

## Quick Start

### 1. Set Up Encryption Key

Generate a secure 256-bit encryption key:

```bash
# Generate a random 32-byte key and encode as base64
openssl rand -base64 32
```

Add to your `.env` file:

```env
ENCRYPTION_MASTER_KEY=your_generated_key_here
```

**CRITICAL SECURITY NOTES:**
- ⚠️ **NEVER commit the encryption key to git**
- ⚠️ Add `ENCRYPTION_MASTER_KEY` to `.gitignore`
- ⚠️ Use different keys for dev/staging/production
- ⚠️ In production, use AWS KMS, Azure Key Vault, or HSM
- ⚠️ Implement key rotation procedures

### 2. Use EncryptedString in Models

Replace `String` with `EncryptedString` for sensitive fields:

```rust
use crate::encryption::EncryptedString;
use diesel::{Queryable, Insertable};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = data_objects)]
pub struct DataObject {
    pub id: Uuid,
    pub creator_id: Uuid,

    // These fields are automatically encrypted in the database
    pub title: EncryptedString,
    pub description: EncryptedString,

    // Regular fields remain unencrypted
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

### 3. Working with EncryptedString

```rust
// Creating an EncryptedString
let title = EncryptedString::from("TOP SECRET//SI//NOFORN");
let title = EncryptedString::new("SECRET Document Title".to_string());

// Reading the decrypted value
println!("Title: {}", title);  // Displays decrypted text
let title_str: &str = title.as_str();

// Converting to String
let title_string: String = title.into_string();

// Checking if empty
if title.is_empty() {
    println!("Title is empty");
}

// Getting length
let len = title.len();
```

### 4. GraphQL Integration

EncryptedString works with async-graphql:

```rust
use async_graphql::*;
use crate::encryption::EncryptedString;

#[derive(SimpleObject)]
pub struct DataObject {
    pub id: Uuid,

    // GraphQL automatically converts EncryptedString to String in responses
    #[graphql(name = "title")]
    pub title: String,  // Use String in GraphQL schema
}

// In your resolver, convert EncryptedString to String
impl DataObject {
    pub async fn title(&self) -> String {
        self.encrypted_title.to_string()
    }
}
```

## Which Fields to Encrypt?

### ✅ SHOULD Encrypt

**High-Value Intelligence Targets:**
- `data_objects.title` - Reveals operation names, targets
- `data_objects.description` - Contains classified content
- `metadata.identifier` - May contain sensitive identifiers
- `metadata.authorization_reference` - Reveals legal authorities
- `authorities.name` - Organization names reveal coalition structure
- `classification_schemas.classification_level` - Capability levels

**Sensitive Metadata:**
- Handling restrictions (e.g., "NOCON", "ORCON")
- Special access program names
- Code words and compartments
- Releasability constraints

### ❌ DO NOT Encrypt

**Operational/Searchable Fields:**
- UUIDs (meaningless without context)
- Timestamps (metadata, low intelligence value)
- Nation codes (public: USA, GBR, FRA)
- Foreign keys (needed for database joins)
- Enum values (domain types, status)

**Reason**: Encrypted fields cannot be efficiently searched or indexed.

## Database Schema Considerations

### Column Type

Encrypted fields are stored as `TEXT` in PostgreSQL:

```sql
CREATE TABLE data_objects (
    id UUID PRIMARY KEY,
    creator_id UUID NOT NULL,

    -- Encrypted fields stored as TEXT (base64-encoded ciphertext)
    title TEXT NOT NULL,
    description TEXT NOT NULL,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

### Storage Overhead

- **Original**: "SECRET" (6 bytes)
- **Encrypted**: "dGVzdA==" (base64 of nonce + ciphertext, ~40-60 bytes)
- **Overhead**: ~5-10x original size

Plan database storage accordingly.

## Migration Strategy

### Option 1: Add New Encrypted Columns (Recommended)

1. Add new encrypted columns
2. Migrate data with encryption
3. Remove old unencrypted columns

```sql
-- Step 1: Add encrypted columns
ALTER TABLE data_objects
ADD COLUMN title_encrypted TEXT,
ADD COLUMN description_encrypted TEXT;

-- Step 2: Application encrypts data and populates new columns
-- (Use Rust migration script)

-- Step 3: Remove old columns
ALTER TABLE data_objects
DROP COLUMN title,
DROP COLUMN description;

-- Step 4: Rename encrypted columns
ALTER TABLE data_objects
RENAME COLUMN title_encrypted TO title,
RENAME COLUMN description_encrypted TO description;
```

### Option 2: In-Place Encryption

```rust
// Migration function to encrypt existing data
use crate::encryption::EncryptedString;
use crate::models::DataObject;

pub fn encrypt_existing_data() -> Result<(), String> {
    let mut conn = database::connection()?;

    // Load all records
    let objects: Vec<DataObject> = data_objects::table
        .load(&mut conn)
        .map_err(|e| e.to_string())?;

    // Re-save each record (triggers encryption)
    for obj in objects {
        diesel::update(data_objects::table.find(obj.id))
            .set(&obj)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

## Searchability Solutions

Encrypted fields cannot be searched directly. Here are solutions:

### Solution 1: Hash Index for Exact Matches

```rust
use sha2::{Sha256, Digest};

// Store hash alongside encrypted data for exact match searching
pub struct DataObject {
    pub title: EncryptedString,
    pub title_hash: String,  // SHA-256 hash for searching
}

// When inserting:
let title = "SECRET Document";
let title_hash = format!("{:x}", Sha256::digest(title.as_bytes()));

// When searching:
let search_hash = format!("{:x}", Sha256::digest(search_term.as_bytes()));
let results = data_objects::table
    .filter(data_objects::title_hash.eq(search_hash))
    .load(&mut conn)?;
```

### Solution 2: Separate Search Index

```rust
// Tokenized search index (less secure, but searchable)
pub struct DataObjectSearchIndex {
    pub data_object_id: Uuid,
    pub token: String,  // Individual searchable words (hashed)
}

// When inserting document:
// - Tokenize title: ["SECRET", "Document"]
// - Hash each token
// - Store in search_index table
// - Actual title stored encrypted in data_objects
```

### Solution 3: Client-Side Decryption for Search

```rust
// Load all records, decrypt in application, filter
// Only viable for small datasets
let all_objects: Vec<DataObject> = data_objects::table.load(&mut conn)?;
let matches: Vec<DataObject> = all_objects
    .into_iter()
    .filter(|obj| obj.title.as_str().contains("SECRET"))
    .collect();
```

## Key Management Best Practices

### Development Environment

```bash
# .env file (DO NOT COMMIT)
ENCRYPTION_MASTER_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
```

### Production Environment

**Option 1: AWS KMS**

```rust
use aws_sdk_kms::Client;

async fn get_encryption_key_from_kms() -> [u8; 32] {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let result = client
        .decrypt()
        .ciphertext_blob(Blob::new(encrypted_key))
        .send()
        .await
        .unwrap();

    // Return decrypted key
    result.plaintext.unwrap().as_ref().try_into().unwrap()
}
```

**Option 2: HashiCorp Vault**

```rust
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

fn get_encryption_key_from_vault() -> [u8; 32] {
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address("https://vault.example.com")
            .token(std::env::var("VAULT_TOKEN").unwrap())
            .build()
            .unwrap()
    ).unwrap();

    let secret: HashMap<String, String> = vaultrs::kv2::read(
        &client,
        "secret/data/encryption",
        "master-key"
    ).unwrap();

    // Decode and return key
    BASE64.decode(secret.get("key").unwrap()).unwrap().try_into().unwrap()
}
```

**Option 3: Hardware Security Module (HSM)**

Use PKCS#11 interface to access keys from HSM without ever exposing the key to application memory.

### Key Rotation Procedure

1. **Generate new key** (KEY_v2)
2. **Add KEY_v2 to environment** alongside KEY_v1
3. **Update encryption module** to try both keys on decryption
4. **Migrate data**: Decrypt with KEY_v1, re-encrypt with KEY_v2
5. **Remove KEY_v1** after all data migrated
6. **Audit**: Verify all data uses KEY_v2

## Security Considerations

### ✅ What This Protects Against

- **Database Backup Theft**: Stolen backups contain only encrypted data
- **SQL Injection**: Even if attacker extracts data, it's encrypted
- **Insider Threat**: DBAs cannot read sensitive fields
- **Physical Server Theft**: Disk encryption + field encryption = defense in depth
- **Cloud Provider Access**: Provider cannot read your encrypted fields
- **Memory Dumps**: Plaintext exists only briefly in application memory

### ⚠️ Limitations

- **Application Compromise**: If attacker controls your app, they have the key
- **Key Theft**: If encryption key is stolen, all data is compromised
- **Side-Channel Attacks**: Timing attacks may leak information
- **Metadata Leakage**: Record counts, relationships still visible

### 🔒 Additional Security Measures

1. **Enable PostgreSQL TDE** (Transparent Data Encryption)
2. **Encrypt network traffic** (TLS 1.3)
3. **Implement audit logging** for all decryption operations
4. **Use separate keys per classification level**
5. **Implement access controls** (who can decrypt what)
6. **Regular security audits** and penetration testing
7. **Monitor for anomalies** (unusual decryption patterns)

## Performance Impact

### Benchmarks

- **Encryption**: ~5-10 microseconds per field (AES-NI hardware acceleration)
- **Decryption**: ~5-10 microseconds per field
- **Database I/O**: ~1-10 milliseconds (dominates total time)

**Conclusion**: Encryption overhead is negligible compared to database I/O.

### Optimization Tips

1. **Batch Operations**: Encrypt/decrypt multiple records in one transaction
2. **Caching**: Cache decrypted values in application (with TTL)
3. **Selective Encryption**: Only encrypt truly sensitive fields
4. **Connection Pooling**: Reuse database connections (already implemented)

## Monitoring and Auditing

### Log Decryption Events

```rust
use log::{info, warn};

impl FromSql<Text, Pg> for EncryptedString {
    fn from_sql(bytes: <Pg as diesel::backend::Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let encrypted_str = <String as FromSql<Text, Pg>>::from_sql(bytes)?;

        // Log decryption event
        info!("Decrypting field at {}", std::time::SystemTime::now());

        let plaintext = decrypt_string(&encrypted_str)
            .map_err(|e| {
                warn!("Decryption failed: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;

        Ok(Self::new(plaintext))
    }
}
```

### Metrics to Track

- **Decryption failures** (may indicate tampering or key issues)
- **Decryption rate** (unusual spikes may indicate data exfiltration)
- **Encryption key access** (who/when/where)
- **Key rotation status**

## Testing

### Unit Tests

```bash
# Run encryption module tests
cargo test encryption::tests
```

### Integration Tests

```rust
#[test]
fn test_encrypted_field_database_roundtrip() {
    let mut conn = establish_connection();

    let original = DataObject {
        id: Uuid::new_v4(),
        title: EncryptedString::from("TOP SECRET//SI"),
        description: EncryptedString::from("Classified operation details"),
        created_at: Utc::now().naive_utc(),
    };

    // Insert into database
    diesel::insert_into(data_objects::table)
        .values(&original)
        .execute(&mut conn)
        .unwrap();

    // Retrieve from database
    let retrieved: DataObject = data_objects::table
        .find(original.id)
        .first(&mut conn)
        .unwrap();

    // Verify decryption worked
    assert_eq!(original.title.as_str(), retrieved.title.as_str());
    assert_eq!(original.description.as_str(), retrieved.description.as_str());
}
```

## Troubleshooting

### "Decryption failed" errors

**Cause**: Invalid ciphertext, wrong key, or data corruption

**Solutions**:
1. Verify `ENCRYPTION_MASTER_KEY` is set correctly
2. Check if you're using the right key for this environment
3. Verify data wasn't corrupted in database
4. Check if data was encrypted with a different key

### Performance degradation

**Cause**: Decrypting many fields in large queries

**Solutions**:
1. Limit query results (pagination)
2. Cache decrypted values in application
3. Only select encrypted fields when needed
4. Consider if all fields need encryption

### Cannot search encrypted fields

**Expected**: This is by design for security

**Solutions**:
1. Implement hash-based exact match search
2. Use separate search index with tokenized data
3. Decrypt on application side for small datasets
4. Consider if field truly needs encryption

## Compliance and Regulations

This encryption implementation helps meet requirements for:

- **NATO Security Classification Guidelines**
- **NIST SP 800-175B** (Key Management)
- **FIPS 140-2** (when using certified crypto modules)
- **GDPR** (data protection at rest)
- **DoD Cybersecurity** (defense in depth)

## Summary

✅ **Implemented**: Field-level AES-256-GCM encryption
✅ **Integrated**: Diesel ORM with transparent encryption/decryption
✅ **Secure**: Authenticated encryption with unique nonces
✅ **Production-Ready**: With proper key management (KMS/HSM)

🔐 **Remember**: Encryption is only as strong as your key management!

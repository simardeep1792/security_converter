# EncryptedString Quick Start Guide

## 5-Minute Setup

### 1. Generate Encryption Key

```bash
openssl rand -base64 32
```

### 2. Add to `.env`

```env
ENCRYPTION_MASTER_KEY=your_generated_key_here
```

### 3. Add to `.gitignore`

```bash
echo "ENCRYPTION_MASTER_KEY" >> .gitignore
```

### 4. Use in Models

```rust
use crate::encryption::EncryptedString;

#[derive(Queryable)]
pub struct YourModel {
    pub id: Uuid,
    pub sensitive_field: EncryptedString,  // 🔒 Automatically encrypted
}
```

### 5. Done!

Data is now automatically encrypted when saved and decrypted when loaded.

---

## Common Operations

### Create

```rust
let encrypted = EncryptedString::from("SECRET DATA");
```

### Read

```rust
println!("{}", encrypted);           // Print decrypted value
let s: &str = encrypted.as_str();   // Get reference
let owned: String = encrypted.into_string(); // Take ownership
```

### Insert to Database

```rust
diesel::insert_into(table)
    .values(&YourModel {
        id: Uuid::new_v4(),
        sensitive_field: EncryptedString::from("SECRET"),
    })
    .execute(&mut conn)?;
```

### Query from Database

```rust
let record: YourModel = table.find(id).first(&mut conn)?;
println!("{}", record.sensitive_field); // Automatically decrypted
```

---

## Fields to Encrypt

✅ **DO Encrypt:**
- Document titles
- Descriptions
- Classification markings
- Organization names
- Authorization references
- Handling restrictions
- Any PII or classified data

❌ **DON'T Encrypt:**
- UUIDs
- Timestamps
- Nation codes (public)
- Foreign keys
- Enums

---

## Security Checklist

- [ ] Generated strong encryption key (32 bytes)
- [ ] Added key to `.env` file
- [ ] Added `.env` to `.gitignore`
- [ ] Using different keys for dev/staging/prod
- [ ] Planned key rotation procedure
- [ ] Considered KMS/HSM for production
- [ ] Enabled PostgreSQL TDE
- [ ] Encrypted database backups
- [ ] Tested encryption/decryption

---

## Production Deployment

### AWS KMS (Recommended)

```rust
// Store encrypted key in environment
// Decrypt using AWS KMS at runtime
// See ENCRYPTION_GUIDE.md for implementation
```

### HashiCorp Vault

```rust
// Fetch key from Vault at startup
// Rotate keys via Vault API
// See ENCRYPTION_GUIDE.md for implementation
```

### Hardware Security Module (HSM)

```rust
// PKCS#11 interface
// Keys never leave HSM
// Best security, highest cost
```

---

## Troubleshooting

**"Decryption failed"**
- Check `ENCRYPTION_MASTER_KEY` is set
- Verify using correct key for this environment
- Check for data corruption

**"Cannot search encrypted fields"**
- This is by design (security feature)
- See ENCRYPTION_GUIDE.md for search solutions

**Performance issues**
- Encryption adds <1% overhead
- Check database query performance first
- Consider caching decrypted values

---

## Learn More

- **Full Guide**: [ENCRYPTION_GUIDE.md](ENCRYPTION_GUIDE.md)
- **Examples**: [ENCRYPTION_EXAMPLE.md](ENCRYPTION_EXAMPLE.md)
- **Implementation**: [src/encryption.rs](src/encryption.rs)

---

## Summary

🔒 **AES-256-GCM** authenticated encryption
🔄 **Transparent** encryption/decryption via Diesel
⚡ **Fast** - hardware-accelerated AES-NI
🛡️ **Secure** - even if database is compromised
📝 **Simple** - just use `EncryptedString` instead of `String`

**Most Important**: Protect your encryption key!

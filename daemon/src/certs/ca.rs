use anyhow::Result;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, KeyUsagePurpose};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tracing::info;

use crate::paths;

pub fn ca_exists() -> bool {
    std::path::Path::new(paths::CA_CERT).exists() && std::path::Path::new(paths::CA_KEY).exists()
}

pub fn ca_cert_path() -> &'static str {
    paths::CA_CERT
}

pub fn ca_key_path() -> &'static str {
    paths::CA_KEY
}

pub fn generate_ca() -> Result<()> {
    if ca_exists() {
        info!("CA already exists, skipping generation");
        return Ok(());
    }

    let mut params = CertificateParams::default();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "LocalDomain Root CA");
    dn.push(DnType::OrganizationName, "LocalDomain");
    params.distinguished_name = dn;
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];

    // Valid for 10 years
    let now = time::OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + time::Duration::days(3650);

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    // Write certificate
    fs::write(paths::CA_CERT, cert.pem())?;

    // Write private key with restricted permissions
    fs::write(paths::CA_KEY, key_pair.serialize_pem())?;
    #[cfg(unix)]
    fs::set_permissions(paths::CA_KEY, fs::Permissions::from_mode(0o600))?;

    info!("Generated root CA certificate");
    Ok(())
}

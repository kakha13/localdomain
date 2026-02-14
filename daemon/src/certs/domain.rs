use anyhow::{Context, Result};
use localdomain_shared::protocol::GenerateCertResult;
use rcgen::{
    CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, KeyPair,
    KeyUsagePurpose, SanType,
};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tracing::info;

use super::ca;
use crate::paths;

pub fn generate_domain_cert(domain: &str) -> Result<GenerateCertResult> {
    // Ensure CA exists
    if !ca::ca_exists() {
        ca::generate_ca()?;
    }

    // Load CA key pair
    let ca_key_pem = fs::read_to_string(ca::ca_key_path()).context("Failed to read CA key")?;
    let ca_key_pair = KeyPair::from_pem(&ca_key_pem)?;

    // Recreate CA params and self-sign to get a Certificate for signing
    let mut ca_params = CertificateParams::default();
    let mut ca_dn = DistinguishedName::new();
    ca_dn.push(DnType::CommonName, "LocalDomain Root CA");
    ca_dn.push(DnType::OrganizationName, "LocalDomain");
    ca_params.distinguished_name = ca_dn;
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];

    let ca_cert = ca_params.self_signed(&ca_key_pair)?;

    // Generate domain certificate
    let mut params = CertificateParams::new(vec![domain.to_string()])?;
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, domain);
    params.distinguished_name = dn;

    params.subject_alt_names = vec![SanType::DnsName(domain.try_into()?)];
    params.use_authority_key_identifier_extension = true;
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ServerAuth);

    let now = time::OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + time::Duration::days(365);

    let domain_key_pair = KeyPair::generate()?;
    let domain_cert = params.signed_by(&domain_key_pair, &ca_cert, &ca_key_pair)?;

    let cert_path = std::path::Path::new(paths::CERTS_DIR)
        .join(format!("{}.crt", domain))
        .to_string_lossy()
        .to_string();
    let key_path = std::path::Path::new(paths::CERTS_DIR)
        .join(format!("{}.key", domain))
        .to_string_lossy()
        .to_string();

    fs::write(&cert_path, domain_cert.pem())?;
    fs::write(&key_path, domain_key_pair.serialize_pem())?;
    #[cfg(unix)]
    fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))?;

    info!("Generated certificate for {}", domain);

    Ok(GenerateCertResult {
        cert_path,
        key_path,
    })
}

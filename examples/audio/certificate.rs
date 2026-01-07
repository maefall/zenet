use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::RootCertStore;
use std::{fs, path::Path};

const CERT_PATH: &str = ".cert.der";
const KEY_PATH: &str = ".key.der";

pub fn load_or_generate_dev_certs<'a>(
) -> anyhow::Result<(rustls::RootCertStore, CertificateDer<'a>, PrivateKeyDer<'a>)> {
    let cert_exists = Path::new(CERT_PATH).exists();
    let key_exists = Path::new(KEY_PATH).exists();

    let (cert_der, key_der) = if cert_exists && key_exists {
        let cert_der = CertificateDer::from(fs::read(CERT_PATH)?);
        let key_der = PrivateKeyDer::try_from(fs::read(KEY_PATH)?).unwrap();

        (cert_der, key_der)
    } else {
        let certified = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
        let cert_der: CertificateDer<'static> = certified.cert.der().clone();
        let key_der = PrivateKeyDer::try_from(certified.signing_key.serialize_der()).unwrap();

        fs::write(CERT_PATH, cert_der.as_ref())?;
        fs::write(KEY_PATH, key_der.secret_der())?;

        (cert_der, key_der)
    };

    let mut root_certs = RootCertStore::empty();

    root_certs.add(cert_der.clone())?;

    Ok((root_certs, cert_der, key_der))
}

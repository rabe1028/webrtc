const SRTP_KEY_LEN: usize = 16;
const SRTP_SALT_LEN: usize = 14;

struct RtcDtlsTransport<Transport> {
    transport: Transport,
}

impl<Transport> RtcDtlsTransport<Transport> {
    fn new(transport: Transport) -> RtcDtlsTransport<Transport> {
        RtcDtlsTransport {
            transport: transport,
        }
    }
}

#[cfg(test)]
mod test {
    use openssl::ec;

    // #[test]
    // fn gen_certificate() {
    //     use openssl::bn::BigNumContext;
    //     use openssl::ec::*;
    //     use openssl::nid::Nid;
    //     use openssl::pkey::PKey;

    //     // get bytes from somewhere, i.e. this will not produce a valid key
    //     let public_key: Vec<u8> = vec![];

    //     // create an EcKey from the binary form of a EcPoint
    //     let group = EcGroup::from_curve_name(Nid::SECP256K1).unwrap();
    //     let mut ctx = BigNumContext::new().unwrap();
    //     let point = EcPoint::from_bytes(&group, &public_key, &mut ctx).unwrap();
    //     let key = EcKey::from_public_key(&group, &point);
    // }
    /*
    DTLS STEPS:
    1. generate private key with SECP256R1 (aiortc, pion)
        ※ SECP256r1 is called prime256v1 in openssl
        // https://stackoverflow.com/questions/41950056/openssl1-1-0-b-is-not-support-secp256r1openssl-ecparam-list-curves
    2. generate x509 certificate
        2.1 hash is SHA256
    */

    #[test]
    fn gen_private_key_test() {
        use chrono::{Date, DateTime, Duration, Utc};
        use openssl::asn1::Asn1Time;
        use openssl::bn::{BigNum, MsbOption};
        use openssl::ec::*;
        use openssl::error::ErrorStack;
        use openssl::hash::MessageDigest;
        use openssl::nid::Nid;
        use openssl::pkey::PKey;
        use openssl::pkey::Private;
        use rand::Rng;

        fn generate_random() -> Result<BigNum, ErrorStack> {
            let mut big = BigNum::new()?;

            // Max random value, a 130-bits integer, i.e 2^130 - 1
            // in pion

            // 20 bytes random number using orandom in aiortc
            // int(binascii.hexlify(data), 16)

            // we create 130-bit odd integer like pion
            big.rand(130, MsbOption::MAYBE_ZERO, true);
            Ok(big)
        }

        //let group = EcGroup::from_curve_name(Nid::SECP256K1).unwrap();
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        let a = ec::EcKey::<Private>::generate(&group).unwrap();
        println!("private_key :: {:?}", a.private_key());

        let mut rng = rand::thread_rng();

        // random str is 128bit(16bytes) hex integer string (aiortc)
        // 16bytes hex string (pion)
        // so 128/4 = 32 char(0~e)
        let random_str = (0..32)
            .map(|_| rng.gen::<u32>() % 16)
            .map(|n| std::char::from_digit(n, 16))
            .collect::<Option<String>>()
            .unwrap();

        // println!("{:?}", random_str);
        let mut x509_name = openssl::x509::X509NameBuilder::new().unwrap();
        x509_name.append_entry_by_text("CN", &random_str).unwrap();
        let x509_name = x509_name.build();

        let mut x509 = openssl::x509::X509::builder().unwrap();
        x509.set_subject_name(&x509_name).unwrap();
        x509.set_issuer_name(&x509_name).unwrap();
        let pubkey = EcKey::from_public_key(&group, a.public_key())
            .and_then(|ec| PKey::from_ec_key(ec))
            .unwrap();
        x509.set_pubkey(&pubkey).unwrap();

        let r = generate_random().and_then(|r| r.to_asn1_integer()).unwrap();
        x509.set_serial_number(&r).unwrap();

        // now - 1 day
        let now: DateTime<Utc> = Utc::now();
        x509.set_not_before(&Asn1Time::from_unix((now - Duration::days(1)).timestamp()).unwrap())
            .unwrap();

        // now + 30 day
        x509.set_not_after(&Asn1Time::from_unix((now + Duration::days(30)).timestamp()).unwrap())
            .unwrap();

        x509.sign(&PKey::from_ec_key(a).unwrap(), MessageDigest::sha256())
            .unwrap();

        let x509 = x509.build();
        println!(
            "{:?}",
            x509.to_pem().unwrap().iter().map(|&s| s as char).collect::<String>()
        );
    }
}

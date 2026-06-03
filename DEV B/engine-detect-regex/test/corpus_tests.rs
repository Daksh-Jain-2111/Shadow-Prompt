use engine_detect_regex::scan_text;

#[cfg(test)]
mod tests {
    use super::*;

    /// Better test macro (checks ANY match, not just index 0)
    macro_rules! test_entities {
        ($name:ident, $entity_type:expr, [$($ex:expr),* $(,)?]) => {
            #[test]
            fn $name() {
                let examples = vec![$($ex),*];

                for (i, input) in examples.iter().enumerate() {
                    let results = scan_text(input);

                    assert!(
                        results.iter().any(|d| d.entity_type == $entity_type),
                        "FAILED: {} Example #{} ({}) not detected",
                        $entity_type,
                        i + 1,
                        input
                    );
                }
            }
        };
    }

    // ---------------- INDIA ----------------

    test_entities!(test_pan, "PAN", [
        "ABCDE1234F","FGHIJ5678K","KLMNO9012L","PQRST3456M","UVWXY7890N",
        "ABCDE1111A","BCDEF2222B","CDEFG3333C","DEFGH4444D","EFGHI5555E",
        "FGHIJ6666F","GHIJK7777G","HIJKL8888H","IJKLM9999I","JKLMN0000J",
        "ZXCVB1357Q","QWERT2468W","ASDFG9753Z","POIUY8642P","LKJHG1212L"
    ]);

    test_entities!(test_aadhaar, "AADHAAR", [
        "123412341234","234523452345","345634563456","456745674567","567856785678",
        "678967896789","789078907890","890189018901","901290129012","112233445566",
        "223344556677","334455667788","445566778899","556677889900","667788990011",
        "778899001122","889900112233","990011223344","101010101010","121212121212"
    ]);

    test_entities!(test_gstin, "GSTIN", [
        "27AAACA1234A1Z5","07AAAAA0000A1Z5","29ABCDE1234F1Z5","33ABCDE1234F1Z5","09ABCDE1234F1Z5",
        "19ABCDE1234F1Z5","24ABCDE1234F1Z5","32ABCDE1234F1Z5","27ABCDE1234F1Z5","10ABCDE1234F1Z5",
        "08ABCDE1234F1Z5","06ABCDE1234F1Z5","05ABCDE1234F1Z5","04ABCDE1234F1Z5","03ABCDE1234F1Z5",
        "02ABCDE1234F1Z5","01ABCDE1234F1Z5","11ABCDE1234F1Z5","12ABCDE1234F1Z5","13ABCDE1234F1Z5"
    ]);

    test_entities!(test_ifsc, "IFSC", [
        "HDFC0001234","ICIC0000001","SBIN0001234","PUNB0123456","KKBK0000001",
        "AXIS0000001","YESB0000001","UTIB0000001","IDFB0000001","CANR0000001",
        "MAHB0000001","UBIN0000001","IBKL0000001","BOFA0000001","HSBC0000001",
        "CITI0000001","SCBL0000001","DBSS0000001","ABNA0000001","BARB0VISHAK"
    ]);

    test_entities!(test_epic, "EPIC", [
        "ABC1234567","DEF2345678","GHI3456789","JKL4567890","MNO5678901",
        "PQR6789012","STU7890123","VWX8901234","YZA9012345","BCD0123456",
        "CDE1234567","EFG2345678","FGH3456789","HIJ4567890","IJK5678901",
        "JKL6789012","KLM7890123","LMN8901234","MNO9012345","NOP0123456"
    ]);

    test_entities!(test_phone, "INDIAN_PHONE", [
        "9876543210","8765432109","7654321098","6543210987","9000000000",
        "+919876543210","+91 9876543210","09876543210","+91-9876543210","919876543210",
        "9123456789","8123456789","7123456789","6123456789","9999988888",
        "9888877777","9777766666","9666655555","9555544444","9444433333"
    ]);

    // ---------------- US ----------------

    test_entities!(test_ssn, "SSN", [
        "123-45-6789","987-65-4321","111-22-3333","222-33-4444","333-44-5555",
        "444-55-6666","555-66-7777","666-77-8888","777-88-9999","888-99-0000",
        "101-10-1001","202-20-2002","303-30-3003","404-40-4004","505-50-5005",
        "606-60-6006","707-70-7007","808-80-8008","909-90-9009","111-11-1111"
    ]);

    test_entities!(test_ein, "EIN", [
        "12-3456789","98-7654321","11-1111111","22-2222222","33-3333333",
        "44-4444444","55-5555555","66-6666666","77-7777777","88-8888888",
        "99-9999999","10-0000001","20-0000002","30-0000003","40-0000004",
        "50-0000005","60-0000006","70-0000007","80-0000008","90-0000009"
    ]);

    // ---------------- UK ----------------

    test_entities!(test_uk_ni, "UK_NI", [
        "AB123456C","CD234567D","EF345678A","GH456789B","JK567890C",
        "LM678901D","NP789012A","QR890123B","ST901234C","UV012345D",
        "WX123456A","YZ234567B","AA345678C","BB456789D","CC567890A",
        "DD678901B","EE789012C","FF890123D","GG901234A","HH012345B"
    ]);

    // ---------------- GLOBAL ----------------

    test_entities!(test_email, "EMAIL", [
        "test@gmail.com","user.name@company.org","admin@domain.net","info@site.io","first.last@co.in",
        "support@github.com","dev@internal.local","sales@shop.com","contact@web.xyz","newsletter@blog.fm",
        "me@personal.me","hiring@startup.vc","legal@firm.law","accounts@bank.com","help@app.app",
        "query@edu.ac.in","test+filter@gmail.com","u.s.e.r@domain.com","12345@numbers.com","x@y.zz"
    ]);

    test_entities!(test_ipv4, "IPV4", [
        "192.168.1.1","10.0.0.1","172.16.0.1","8.8.8.8","127.0.0.1",
        "255.255.255.255","1.1.1.1","123.45.67.89","99.99.99.99","200.200.200.200",
        "50.60.70.80","11.22.33.44","66.77.88.99","101.102.103.104","150.160.170.180",
        "201.202.203.204","210.220.230.240","5.5.5.5","9.9.9.9","12.34.56.78"
    ]);

    test_entities!(test_url, "URL", [
        "https://google.com","http://example.com","https://github.com","https://openai.com","http://test.org",
        "https://sub.domain.com","https://site.io/path","http://abc.xyz","https://docs.rs","https://rust-lang.org",
        "https://news.site","http://localhost:8080","https://my.app","https://hello.world","https://api.service.com",
        "http://random.site","https://blog.dev","https://learn.io","https://xyz.net","https://abc.in"
    ]);

    test_entities!(test_card, "CREDIT_CARD", [
        "4111111111111111","5555555555554444","378282246310005","6011000000000000","4222222222222",
        "4000056655665556","5200828282828210","4539578763621486","4716108999716531","6011111111111117",
        "3530111333300000","3566002020360505","30569309025904","38520000023237","6011000990139424",
        "4000000000000002","4007000000027","4222222222222222","5555555555554444","4111111111111111"
    ]);

    #[test]
    fn test_zero_false_positives() {
        let text = "The quick brown fox jumps over the lazy dog.";
        let results = scan_text(text);
        assert!(results.is_empty(), "False positive detected");
    }
}

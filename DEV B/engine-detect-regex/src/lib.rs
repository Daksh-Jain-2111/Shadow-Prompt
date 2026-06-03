use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // INDIA
    static ref AADHAAR_RE: Regex = Regex::new(r"\b\d{4}\s?\d{4}\s?\d{4}\b").unwrap();
    static ref PAN_RE: Regex = Regex::new(r"\b[A-Z]{5}[0-9]{4}[A-Z]\b").unwrap();
    static ref IFSC_RE: Regex = Regex::new(r"\b[A-Z]{4}0[A-Z0-9]{6}\b").unwrap();
    static ref GSTIN_RE: Regex = Regex::new(r"\b\d{2}[A-Z]{5}[0-9]{4}[A-Z][1-9A-Z]Z[0-9A-Z]\b").unwrap();
    static ref EPIC_RE: Regex = Regex::new(r"\b[A-Z]{3}[0-9]{7}\b").unwrap();
    static ref IN_PHONE_RE: Regex = Regex::new(r"(?:\+91[-\s]?|0)?[6-9]\d{9}\b").unwrap();

    // US
    static ref SSN_RE: Regex = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap();
    static ref EIN_RE: Regex = Regex::new(r"\b\d{2}-\d{7}\b").unwrap();

    // UK
    static ref UK_NI_RE: Regex = Regex::new(r"\b[A-Z]{2}\d{6}[A-D]\b").unwrap();

    // GLOBAL
    static ref IBAN_RE: Regex = Regex::new(r"\b[A-Z]{2}\d{2}[A-Z0-9]{11,30}\b").unwrap();
    static ref CARD_RE: Regex = Regex::new(r"\b((?:\d{4}[- ]?){3}\d{4}|\d{13,19})\b").unwrap();

    static ref EMAIL_RE: Regex = Regex::new(r#"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b"#).unwrap();
    static ref IPV4_RE: Regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
    static ref IPV6_RE: Regex = Regex::new(r"\b([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b").unwrap();
    static ref URL_RE: Regex = Regex::new(r"https?://[^\s/$.?#].[^\s]*").unwrap();
}

#[derive(Debug, Clone)]
pub struct Detection {
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub is_valid: bool,
}

pub fn scan_text(text: &str) -> Vec<Detection> {
    let mut detections = Vec::new();

    // INDIA
    for mat in AADHAAR_RE.find_iter(text) {
        detections.push(create_detection(
            "AADHAAR",
            mat,
            validate_verhoeff(mat.as_str()),
        ));
    }

    for mat in PAN_RE.find_iter(text) {
        detections.push(create_detection("PAN", mat, true));
    }

    for mat in IFSC_RE.find_iter(text) {
        detections.push(create_detection("IFSC", mat, true));
    }

    for mat in GSTIN_RE.find_iter(text) {
        detections.push(create_detection("GSTIN", mat, true));
    }

    for mat in EPIC_RE.find_iter(text) {
        detections.push(create_detection("EPIC", mat, true));
    }

    for mat in IN_PHONE_RE.find_iter(text) {
        detections.push(create_detection("INDIAN_PHONE", mat, true));
    }

    // US
    for mat in SSN_RE.find_iter(text) {
        detections.push(create_detection("SSN", mat, true));
    }

    for mat in EIN_RE.find_iter(text) {
        detections.push(create_detection("EIN", mat, true));
    }

    // UK
    for mat in UK_NI_RE.find_iter(text) {
        detections.push(create_detection("UK_NI", mat, true));
    }

    // GLOBAL
    for mat in IBAN_RE.find_iter(text) {
        detections.push(create_detection("IBAN", mat, true));
    }

    for mat in CARD_RE.find_iter(text) {
        detections.push(create_detection(
            "CREDIT_CARD",
            mat,
            validate_luhn(mat.as_str()),
        ));
    }

    for mat in EMAIL_RE.find_iter(text) {
        detections.push(create_detection("EMAIL", mat, true));
    }

    for mat in IPV4_RE.find_iter(text) {
        detections.push(create_detection("IPV4", mat, true));
    }

    for mat in IPV6_RE.find_iter(text) {
        detections.push(create_detection("IPV6", mat, true));
    }

    for mat in URL_RE.find_iter(text) {
        detections.push(create_detection("URL", mat, true));
    }

    detections
}

pub fn scan_valid_text(text: &str) -> Vec<Detection> {
    scan_text(text).into_iter().filter(|d| d.is_valid).collect()
}

fn create_detection(label: &str, mat: regex::Match, is_valid: bool) -> Detection {
    Detection {
        entity_type: label.to_string(),
        start: mat.start(),
        end: mat.end(),
        value: mat.as_str().to_string(),
        is_valid,
    }
}

// ---------------- VALIDATIONS ----------------

fn validate_luhn(n: &str) -> bool {
    let digits: Vec<u32> = n.chars()
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.is_empty() {
        return false;
    }

    let mut sum = 0;
    let mut second = false;

    for &d in digits.iter().rev() {
        let mut val = d;
        if second {
            val *= 2;
            if val > 9 {
                val -= 9;
            }
        }
        sum += val;
        second = !second;
    }

    sum % 10 == 0
}

fn validate_verhoeff(n: &str) -> bool {
    let d = [
        [0,1,2,3,4,5,6,7,8,9],[1,2,3,4,0,6,7,8,9,5],
        [2,3,4,0,1,7,8,9,5,6],[3,4,0,1,2,8,9,5,6,7],
        [4,0,1,2,3,9,5,6,7,8],[5,9,8,7,6,0,4,3,2,1],
        [6,5,9,8,7,1,0,4,3,2],[7,6,5,9,8,2,1,0,4,3],
        [8,7,6,5,9,3,2,1,0,4],[9,8,7,6,5,4,3,2,1,0],
    ];

    let p = [
        [0,1,2,3,4,5,6,7,8,9],[1,5,7,6,2,8,3,0,9,4],
        [5,8,0,3,7,9,6,1,4,2],[8,9,1,6,0,4,3,5,2,7],
        [9,4,5,3,1,2,6,8,7,0],[4,2,8,6,5,7,3,9,0,1],
        [2,7,9,3,8,0,6,4,1,5],[7,0,4,6,9,1,3,2,5,8],
    ];

    let mut c = 0;

    let digits: Vec<usize> = n.chars()
        .filter_map(|x| x.to_digit(10).map(|d| d as usize))
        .collect();

    for (i, &digit) in digits.iter().rev().enumerate() {
        c = d[c][p[i % 8][digit]];
    }

    c == 0
}

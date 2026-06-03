# =========================================================
# SHADOWPROMPT - INDIAN REGEX VALIDATOR ENGINE
# =========================================================

import re

# =========================================================
# VERHOEFF TABLES (AADHAAR CHECKSUM)
# =========================================================

d_table = [
    [0,1,2,3,4,5,6,7,8,9],
    [1,2,3,4,0,6,7,8,9,5],
    [2,3,4,0,1,7,8,9,5,6],
    [3,4,0,1,2,8,9,5,6,7],
    [4,0,1,2,3,9,5,6,7,8],
    [5,9,8,7,6,0,4,3,2,1],
    [6,5,9,8,7,1,0,4,3,2],
    [7,6,5,9,8,2,1,0,4,3],
    [8,7,6,5,9,3,2,1,0,4],
    [9,8,7,6,5,4,3,2,1,0]
]

p_table = [
    [0,1,2,3,4,5,6,7,8,9],
    [1,5,7,6,2,8,3,0,9,4],
    [5,8,0,3,7,9,6,1,4,2],
    [8,9,1,6,0,4,3,5,2,7],
    [9,4,5,3,1,2,6,8,7,0],
    [4,2,8,6,5,7,3,9,0,1],
    [2,7,9,3,8,0,6,4,1,5],
    [7,0,4,6,9,1,3,2,5,8]
]

# =========================================================
# REGEX PATTERNS
# =========================================================

AADHAAR_REGEX = r"\b\d{4}[- ]?\d{4}[- ]?\d{4}\b"

PAN_REGEX = r"\b[A-Z]{5}[0-9]{4}[A-Z]\b"

IFSC_REGEX = r"\b[A-Z]{4}0[A-Z0-9]{6}\b"

GSTIN_REGEX = (
    r"\b\d{2}[A-Z]{5}\d{4}[A-Z]"
    r"[1-9A-Z]Z[0-9A-Z]\b"
)

PHONE_REGEX = (
    r"\b(?:\+91[- ]?)?[6-9]\d{9}\b"
)

# =========================================================
# VERHOEFF VALIDATOR
# =========================================================

def validate_verhoeff(number):

    c = 0

    reversed_digits = list(
        map(int, reversed(number))
    )

    for i, digit in enumerate(reversed_digits):

        c = d_table[c][
            p_table[i % 8][digit]
        ]

    return c == 0


# =========================================================
# AADHAAR DETECTOR
# =========================================================

def detect_aadhaar(text):

    matches = re.findall(
        AADHAAR_REGEX,
        text
    )

    valid_matches = []

    for match in matches:

        clean = re.sub(
            r"\D",
            "",
            match
        )

        if len(clean) == 12:

            if validate_verhoeff(clean):

                valid_matches.append(clean)

    return valid_matches


# =========================================================
# PAN DETECTOR
# =========================================================

def detect_pan(text):

    return re.findall(
        PAN_REGEX,
        text
    )


# =========================================================
# IFSC DETECTOR
# =========================================================

def detect_ifsc(text):

    return re.findall(
        IFSC_REGEX,
        text
    )


# =========================================================
# GSTIN DETECTOR
# =========================================================

def detect_gstin(text):

    return re.findall(
        GSTIN_REGEX,
        text
    )


# =========================================================
# PHONE DETECTOR
# =========================================================

def detect_phone(text):

    return re.findall(
        PHONE_REGEX,
        text
    )


# =========================================================
# TEST CASES
# =========================================================

test_cases = [

    {
        "text":
            "My Aadhaar is 331579974693",
        "expect":
            "AADHAAR"
    },

    {
        "text":
            "PAN is ABCDE1234F",
        "expect":
            "PAN"
    },

    {
        "text":
            "IFSC code SBIN0001234",
        "expect":
            "IFSC"
    },

    {
        "text":
            "GSTIN 27ABCDE1234F1Z5",
        "expect":
            "GSTIN"
    },

    {
        "text":
            "Phone number 9876543210",
        "expect":
            "PHONE"
    },

    {
        "text":
            "Invalid Aadhaar 111111111111",
        "expect":
            "INVALID"
    },

    {
        "text":
            "PAN WRONG123",
        "expect":
            "INVALID"
    },

    {
        "text":
            "Phone 12345",
        "expect":
            "INVALID"
    },

    {
        "text":
            "GST fake 12ABCDE1234F1Z",
        "expect":
            "INVALID"
    },

    {
        "text":
            "IFSC TEST00012",
        "expect":
            "INVALID"
    }
]

# =========================================================
# TEST RUNNER
# =========================================================

print("\n===================================")
print("SHADOWPROMPT REGEX VALIDATOR TEST")
print("===================================\n")

passed = 0

for idx, case in enumerate(
    test_cases,
    start=1
):

    text = case["text"]

    expected = case["expect"]

    aadhaar = detect_aadhaar(text)

    pan = detect_pan(text)

    ifsc = detect_ifsc(text)

    gstin = detect_gstin(text)

    phone = detect_phone(text)

    detected = []

    if aadhaar:
        detected.append("AADHAAR")

    if pan:
        detected.append("PAN")

    if ifsc:
        detected.append("IFSC")

    if gstin:
        detected.append("GSTIN")

    if phone:
        detected.append("PHONE")

    # =========================================
    # VALIDATION
    # =========================================

    success = False

    if expected == "INVALID":

        if len(detected) == 0:
            success = True

    else:

        if expected in detected:
            success = True

    # =========================================
    # OUTPUT
    # =========================================

    print(f"TEST {idx}")

    print(f"Input: {text}")

    print(f"Detected: {detected}")

    print(
        "PASS ✅"
        if success
        else "FAIL ❌"
    )

    print("-----------------------------------")

    if success:
        passed += 1


# =========================================================
# FINAL REPORT
# =========================================================

print("\n===================================")
print("FINAL RESULT")
print("===================================\n")

print(f"Passed: {passed}/10")

if passed == 10:

    print("\n✅ Deliverable Completed")

else:

    print("\n❌ Deliverable NOT Fully Complete")


# =========================================================
# MANUAL TESTS
# =========================================================

print("\n===================================")
print("MANUAL TEST")
print("===================================\n")

while True:

    user_input = input(
        "\nEnter text (or type exit): "
    )

    if user_input.lower() == "exit":
        break

    print("\nDetected Entities:\n")

    print(
        "AADHAAR:",
        detect_aadhaar(user_input)
    )

    print(
        "PAN:",
        detect_pan(user_input)
    )

    print(
        "IFSC:",
        detect_ifsc(user_input)
    )

    print(
        "GSTIN:",
        detect_gstin(user_input)
    )

    print(
        "PHONE:",
        detect_phone(user_input)
    )

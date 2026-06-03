import json
import random
import string

# =========================================================
# CONFIG
# =========================================================

NUM_SAMPLES = 500

OUTPUT_FILE = "data/synthetic/indian_ner_dataset.json"

# =========================================================
# INDIAN DATA
# =========================================================

FIRST_NAMES = [
    "Priya",
    "Rahul",
    "Ananya",
    "Arjun",
    "Sneha",
    "Vikram",
    "Kavya",
    "Rohan",
    "Amit",
    "Neha"
]

LAST_NAMES = [
    "Sharma",
    "Kapoor",
    "Iyer",
    "Mehta",
    "Singh",
    "Reddy",
    "Das",
    "Nair",
    "Jain",
    "Patel"
]

BANKS = [
    "HDFC Bank",
    "ICICI Bank",
    "State Bank of India",
    "Axis Bank",
    "Kotak Mahindra Bank"
]

CITIES = [
    "Mumbai",
    "Delhi",
    "Bangalore",
    "Pune",
    "Hyderabad",
    "Chennai"
]

COMPANIES = [
    "Infosys",
    "TCS",
    "Wipro",
    "Reliance",
    "Flipkart"
]

# =========================================================
# PAN GENERATOR
# =========================================================

def generate_pan():

    letters1 = ''.join(random.choices(string.ascii_uppercase, k=5))

    digits = ''.join(random.choices(string.digits, k=4))

    letters2 = random.choice(string.ascii_uppercase)

    return f"{letters1}{digits}{letters2}"


# =========================================================
# FAKE AADHAAR GENERATOR
# NOTE:
# This does NOT create verified Verhoeff-valid Aadhaar.
# For production use checksum validation separately.
# =========================================================

def generate_aadhaar():

    parts = []

    for _ in range(3):
        parts.append(
            ''.join(random.choices(string.digits, k=4))
        )

    return "-".join(parts)


# =========================================================
# TOKEN + LABEL HELPERS
# =========================================================

def add_person(tokens, labels, first, last):

    tokens.extend([first, last])

    labels.extend(["B-PER", "I-PER"])


def add_org(tokens, labels, org_name):

    org_tokens = org_name.split()

    for i, token in enumerate(org_tokens):

        tokens.append(token)

        if i == 0:
            labels.append("B-ORG")
        else:
            labels.append("I-ORG")


def add_loc(tokens, labels, city):

    tokens.append(city)

    labels.append("B-LOC")


# =========================================================
# TEMPLATE GENERATORS
# =========================================================

def template_bank_employee():

    first = random.choice(FIRST_NAMES)
    last = random.choice(LAST_NAMES)

    bank = random.choice(BANKS)

    city = random.choice(CITIES)

    tokens = []
    labels = []

    add_person(tokens, labels, first, last)

    sentence = [
        "works",
        "at"
    ]

    tokens.extend(sentence)

    labels.extend(["O", "O"])

    add_org(tokens, labels, bank)

    tokens.append("in")
    labels.append("O")

    add_loc(tokens, labels, city)

    return {
        "tokens": tokens,
        "ner_tags": labels
    }


def template_pan():

    first = random.choice(FIRST_NAMES)
    last = random.choice(LAST_NAMES)

    pan = generate_pan()

    tokens = []
    labels = []

    add_person(tokens, labels, first, last)

    extra = [
        "submitted",
        "PAN",
        pan
    ]

    tokens.extend(extra)

    labels.extend([
        "O",
        "O",
        "O"
    ])

    return {
        "tokens": tokens,
        "ner_tags": labels
    }


def template_aadhaar():

    first = random.choice(FIRST_NAMES)
    last = random.choice(LAST_NAMES)

    aadhaar = generate_aadhaar()

    tokens = []
    labels = []

    add_person(tokens, labels, first, last)

    extra = [
        "has",
        "Aadhaar",
        aadhaar
    ]

    tokens.extend(extra)

    labels.extend([
        "O",
        "O",
        "O"
    ])

    return {
        "tokens": tokens,
        "ner_tags": labels
    }


def template_company():

    first = random.choice(FIRST_NAMES)
    last = random.choice(LAST_NAMES)

    company = random.choice(COMPANIES)

    tokens = []
    labels = []

    add_person(tokens, labels, first, last)

    extra = [
        "joined"
    ]

    tokens.extend(extra)

    labels.extend(["O"])

    add_org(tokens, labels, company)

    return {
        "tokens": tokens,
        "ner_tags": labels
    }


# =========================================================
# TEMPLATE POOL
# =========================================================

TEMPLATES = [
    template_bank_employee,
    template_pan,
    template_aadhaar,
    template_company
]

# =========================================================
# GENERATE DATASET
# =========================================================

dataset = []

for _ in range(NUM_SAMPLES):

    generator = random.choice(TEMPLATES)

    sample = generator()

    dataset.append(sample)

# =========================================================
# SAVE DATASET
# =========================================================

with open(OUTPUT_FILE, "w", encoding="utf-8") as f:

    json.dump(
        dataset,
        f,
        indent=2,
        ensure_ascii=False
    )

# =========================================================
# PREVIEW
# =========================================================

print("\nDataset Generated Successfully!\n")

print(f"Saved to: {OUTPUT_FILE}")

print("\nSample:\n")

print(json.dumps(dataset[0], indent=2))
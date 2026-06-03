# =========================================================
# GENERATE SYNTHETIC MONEY + DATE NER DATASET
# =========================================================

import json
import random

# =========================================================
# CONFIG
# =========================================================

NUM_SAMPLES = 500

OUTPUT_FILE = (
    "data/synthetic/"
    "money_date_dataset.json"
)

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
    "Hyderabad"
]

DATES = [
    "12 January 2025",
    "5 March 2024",
    "18 August 2026",
    "01/02/2025",
    "15-09-2024",
    "7 July 2023"
]

MONEY_VALUES = [
    "₹50,000",
    "₹1 lakh",
    "₹2,50,000",
    "₹999",
    "₹75 crore",
    "₹10,500"
]

# =========================================================
# HELPER FUNCTIONS
# =========================================================

def add_person(tokens, labels, first, last):

    tokens.extend([first, last])

    labels.extend([
        "B-PER",
        "I-PER"
    ])


def add_org(tokens, labels, org):

    org_tokens = org.split()

    for i, token in enumerate(org_tokens):

        tokens.append(token)

        if i == 0:
            labels.append("B-ORG")

        else:
            labels.append("I-ORG")


def add_money(tokens, labels, money):

    money_tokens = money.split()

    for i, token in enumerate(money_tokens):

        tokens.append(token)

        if i == 0:
            labels.append("B-MONEY")

        else:
            labels.append("I-MONEY")


def add_date(tokens, labels, date):

    date_tokens = date.split()

    for i, token in enumerate(date_tokens):

        tokens.append(token)

        if i == 0:
            labels.append("B-DATE")

        else:
            labels.append("I-DATE")


# =========================================================
# TEMPLATE 1
# =========================================================

def template_transfer():

    first = random.choice(FIRST_NAMES)

    last = random.choice(LAST_NAMES)

    bank = random.choice(BANKS)

    money = random.choice(MONEY_VALUES)

    date = random.choice(DATES)

    tokens = []

    labels = []

    add_person(
        tokens,
        labels,
        first,
        last
    )

    tokens.extend([
        "transferred"
    ])

    labels.extend([
        "O"
    ])

    add_money(
        tokens,
        labels,
        money
    )

    tokens.extend([
        "through"
    ])

    labels.extend([
        "O"
    ])

    add_org(
        tokens,
        labels,
        bank
    )

    tokens.extend([
        "on"
    ])

    labels.extend([
        "O"
    ])

    add_date(
        tokens,
        labels,
        date
    )

    return {
        "tokens": tokens,
        "ner_tags": labels
    }


# =========================================================
# TEMPLATE 2
# =========================================================

def template_salary():

    first = random.choice(FIRST_NAMES)

    last = random.choice(LAST_NAMES)

    money = random.choice(MONEY_VALUES)

    date = random.choice(DATES)

    tokens = []

    labels = []

    add_person(
        tokens,
        labels,
        first,
        last
    )

    tokens.extend([
        "received",
        "salary",
        "of"
    ])

    labels.extend([
        "O",
        "O",
        "O"
    ])

    add_money(
        tokens,
        labels,
        money
    )

    tokens.extend([
        "on"
    ])

    labels.extend([
        "O"
    ])

    add_date(
        tokens,
        labels,
        date
    )

    return {
        "tokens": tokens,
        "ner_tags": labels
    }


# =========================================================
# TEMPLATE 3
# =========================================================

def template_loan():

    first = random.choice(FIRST_NAMES)

    last = random.choice(LAST_NAMES)

    bank = random.choice(BANKS)

    money = random.choice(MONEY_VALUES)

    date = random.choice(DATES)

    tokens = []

    labels = []

    add_person(
        tokens,
        labels,
        first,
        last
    )

    tokens.extend([
        "applied",
        "for",
        "loan",
        "of"
    ])

    labels.extend([
        "O",
        "O",
        "O",
        "O"
    ])

    add_money(
        tokens,
        labels,
        money
    )

    tokens.extend([
        "from"
    ])

    labels.extend([
        "O"
    ])

    add_org(
        tokens,
        labels,
        bank
    )

    tokens.extend([
        "on"
    ])

    labels.extend([
        "O"
    ])

    add_date(
        tokens,
        labels,
        date
    )

    return {
        "tokens": tokens,
        "ner_tags": labels
    }


# =========================================================
# TEMPLATE POOL
# =========================================================

TEMPLATES = [
    template_transfer,
    template_salary,
    template_loan
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

with open(
    OUTPUT_FILE,
    "w",
    encoding="utf-8"
) as f:

    json.dump(
        dataset,
        f,
        indent=2,
        ensure_ascii=False
    )

# =========================================================
# PREVIEW
# =========================================================

print("\n===================================")
print("DATASET GENERATED")
print("===================================\n")

print(f"Saved To: {OUTPUT_FILE}")

print("\nSample:\n")

print(json.dumps(
    dataset[0],
    indent=2,
    ensure_ascii=False
))
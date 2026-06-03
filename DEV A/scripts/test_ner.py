from transformers import pipeline

ner = pipeline(
    "ner",
    model="dslim/distilbert-NER",
    aggregation_strategy="simple"
)

text = """
Priya Sharma from HDFC Bank sent ₹85,000 to Raj Mehta.
"""

results = ner(text)

for r in results:
    print(r)
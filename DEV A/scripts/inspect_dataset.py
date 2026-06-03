import pandas as pd
import numpy as np
conll = pd.read_parquet(
    "data/raw/conull2003/train.parquet"
)

wikiann = pd.read_parquet(
    "data/raw/wikiann/en/train.parquet"
)

print("CONLL")
print(conll.head())
print(conll.columns)

print("\nWIKIANN")
print(wikiann.head())
print(wikiann.columns)
# =========================================================
# SHADOWPROMPT ONNX RUNTIME TEST
# =========================================================

import numpy as np
import onnxruntime as ort

# =========================================================
# LOAD ONNX MODEL
# =========================================================

print("\n===================================")
print("LOADING ONNX MODEL")
print("===================================\n")

session = ort.InferenceSession(
    "models/model.onnx"
)

print("✅ ONNX model loaded successfully!")

# =========================================================
# PRINT INPUTS
# =========================================================

print("\n===================================")
print("MODEL INPUTS")
print("===================================\n")

for inp in session.get_inputs():

    print("Name :", inp.name)

    print("Shape:", inp.shape)

    print("Type :", inp.type)

    print("-----------------------------------")

# =========================================================
# PRINT OUTPUTS
# =========================================================

print("\n===================================")
print("MODEL OUTPUTS")
print("===================================\n")

for out in session.get_outputs():

    print("Name :", out.name)

    print("Shape:", out.shape)

    print("Type :", out.type)

    print("-----------------------------------")

# =========================================================
# CREATE DUMMY INPUT
# =========================================================

print("\n===================================")
print("CREATING DUMMY INPUT")
print("===================================\n")

input_ids = np.ones(
    (1, 128),
    dtype=np.int64
)

attention_mask = np.ones(
    (1, 128),
    dtype=np.int64
)

inputs = {

    "input_ids": input_ids,

    "attention_mask": attention_mask
}

print("✅ Dummy input created!")

# =========================================================
# RUN INFERENCE
# =========================================================

print("\n===================================")
print("RUNNING ONNX INFERENCE")
print("===================================\n")

outputs = session.run(
    None,
    inputs
)

print("✅ ONNX Runtime inference successful!")

# =========================================================
# OUTPUT DETAILS
# =========================================================

print("\n===================================")
print("OUTPUT DETAILS")
print("===================================\n")

print("Number of outputs:", len(outputs))

print("\nOutput shape:")

print(outputs[0].shape)

print("\nOutput dtype:")

print(outputs[0].dtype)

# =========================================================
# SAMPLE OUTPUT
# =========================================================

print("\n===================================")
print("SAMPLE OUTPUT VALUES")
print("===================================\n")

print(outputs[0][0][0][:10])

# =========================================================
# FINAL STATUS
# =========================================================

print("\n===================================")
print("FINAL STATUS")
print("===================================\n")

print("✅ ONNX file loaded")
print("✅ Input tensors accepted")
print("✅ ONNX Runtime inference passed")
print("✅ Output tensor generated")

print("\nDELIVERABLE COMPLETED SUCCESSFULLY!\n")
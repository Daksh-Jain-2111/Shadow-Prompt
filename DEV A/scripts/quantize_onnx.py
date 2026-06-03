# =========================================================
# SHADOWPROMPT INT8 QUANTIZATION
# =========================================================

from onnxruntime.quantization import (

    quantize_dynamic,

    QuantType
)

# =========================================================
# PATHS
# =========================================================

INPUT_MODEL = "models/model.onnx"

OUTPUT_MODEL = "models/model_int8.onnx"

# =========================================================
# QUANTIZATION
# =========================================================

print("\n===================================")
print("STARTING INT8 QUANTIZATION")
print("===================================\n")

quantize_dynamic(

    model_input=INPUT_MODEL,

    model_output=OUTPUT_MODEL,

    weight_type=QuantType.QInt8
)

print("\n✅ INT8 quantization complete!")

print(f"\nSaved model: {OUTPUT_MODEL}")

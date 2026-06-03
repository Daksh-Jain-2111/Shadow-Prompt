# =========================================================
# ONNX -> TFLITE CONVERSION
# =========================================================

import tensorflow as tf
import onnx

from onnx_tf.backend import prepare

# =========================================================
# LOAD ONNX
# =========================================================

print("\nLoading ONNX model...\n")

onnx_model = onnx.load(
    "models/model_int8.onnx"
)

# =========================================================
# CONVERT TO TENSORFLOW
# =========================================================

print("\nConverting ONNX -> TensorFlow...\n")

tf_rep = prepare(onnx_model)

TF_MODEL_PATH = "models/tf_model"

tf_rep.export_graph(TF_MODEL_PATH)

print("\n✅ TensorFlow export successful!")

# =========================================================
# LOAD SAVEDMODEL
# =========================================================

print("\nLoading TensorFlow SavedModel...\n")

converter = tf.lite.TFLiteConverter.from_saved_model(
    TF_MODEL_PATH
)

# =========================================================
# TFLITE OPTIMIZATION
# =========================================================

converter.optimizations = [
    tf.lite.Optimize.DEFAULT
]

# =========================================================
# CONVERT
# =========================================================

print("\nConverting -> TFLite...\n")

tflite_model = converter.convert()

# =========================================================
# SAVE
# =========================================================

TFLITE_PATH = "models/model.tflite"

with open(TFLITE_PATH, "wb") as f:

    f.write(tflite_model)

print("\n✅ TFLite conversion successful!")

print(f"\nSaved: {TFLITE_PATH}")
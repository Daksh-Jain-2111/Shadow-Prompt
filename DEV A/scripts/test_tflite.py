import numpy as np
import tensorflow as tf

# =========================================================
# LOAD TFLITE
# =========================================================

print("\nLoading TFLite model...\n")

interpreter = tf.lite.Interpreter(
    model_path="models/model.tflite"
)

interpreter.allocate_tensors()

# =========================================================
# INPUT DETAILS
# =========================================================

input_details = interpreter.get_input_details()

output_details = interpreter.get_output_details()

print("\nInput Details:\n")

print(input_details)

print("\nOutput Details:\n")

print(output_details)

# =========================================================
# DUMMY INPUT
# =========================================================

input_ids = np.ones(
    (1, 128),
    dtype=np.int64
)

attention_mask = np.ones(
    (1, 128),
    dtype=np.int64
)

# =========================================================
# SET INPUTS
# =========================================================

interpreter.set_tensor(

    input_details[0]["index"],

    input_ids
)

interpreter.set_tensor(

    input_details[1]["index"],

    attention_mask
)

# =========================================================
# RUN
# =========================================================

print("\nRunning TFLite inference...\n")

interpreter.invoke()

output = interpreter.get_tensor(
    output_details[0]["index"]
)

print("\n✅ TFLite inference successful!")

print("\nOutput shape:\n")

print(output.shape)
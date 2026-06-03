# =========================================================
# SHADOWPROMPT ONNX BENCHMARK
# =========================================================

import time
import numpy as np
import onnxruntime as ort

# =========================================================
# TEST BOTH MODELS
# =========================================================

MODELS = {

    "FP32":
        "models/model.onnx",

    "INT8":
        "models/model_int8.onnx"
}

# =========================================================
# DUMMY INPUT
# =========================================================

inputs = {

    "input_ids":
        np.ones((1, 128), dtype=np.int64),

    "attention_mask":
        np.ones((1, 128), dtype=np.int64)
}

# =========================================================
# BENCHMARK
# =========================================================

for name, path in MODELS.items():

    print("\n===================================")

    print(f"TESTING {name}")

    print("===================================\n")

    session = ort.InferenceSession(path)

    # WARMUP
    for _ in range(5):

        session.run(None, inputs)

    # TIMING
    runs = 20

    start = time.time()

    for _ in range(runs):

        outputs = session.run(
            None,
            inputs
        )

    end = time.time()

    avg_latency = (
        (end - start) / runs
    ) * 1000

    print(
        f"Average Latency: "
        f"{avg_latency:.2f} ms"
    )

    print(
        f"Output Shape: "
        f"{outputs[0].shape}"
    )
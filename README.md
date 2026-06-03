# 🛡️ ShadowPrompt

**Privacy-First AI Gateway for Secure Prompt Processing**

---

## 🚀 Overview

ShadowPrompt is a privacy-focused middleware that detects and masks Personally Identifiable Information (PII) before sending user data to AI systems.

It acts as a **secure layer between users and AI models**, ensuring sensitive data like names, emails, phone numbers, and IDs are never exposed.

---

## 🎯 Problem

Modern AI tools require user data to generate responses. However:

* Sensitive information is often sent directly to AI APIs
* No control over how data is stored or processed
* High risk of privacy leaks and misuse

---

## 💡 Solution

ShadowPrompt introduces a **Zero-Trust pipeline** that:

1. Intercepts user input
2. Detects sensitive data using:

   * Regex-based rules (fast detection)
   * ML-based Named Entity Recognition (NER)
3. Masks or anonymizes PII
4. Sends sanitized data to AI models

---

## 🧠 Architecture

```
User Input
   ↓
STM (In-Memory Storage)
   ↓
Regex Detection Engine
   ↓
ML NER Model (DistilBERT)
   ↓
Masking Engine
   ↓
Sanitized Output → AI Model
```

---

## 🔐 Core Principles

* **Zero Trust**: No sensitive data is trusted or stored permanently
* **In-Memory Processing**: No disk writes for user data
* **Data Minimization**: Only necessary information is retained
* **Security by Design**: Memory is wiped after processing

---

## ⚙️ Tech Stack

### Backend (Dev B)

* Rust
* Regex Engine
* Zeroization (`zeroize` crate)
* REST API (Actix Web)

### AI Layer (Dev A)

* Python
* PyTorch
* HuggingFace Transformers
* DistilBERT (NER Model)
* ONNX (for optimization)

---

## 📦 Features

* ✅ Real-time PII detection
* ✅ Hybrid detection (Regex + ML)
* ✅ Secure session-based memory (STM)
* ✅ Data masking and anonymization
* ✅ API-ready architecture

---

## 🧩 Modules

### 1. STM (Short-Term Memory)

* Stores session data temporarily
* Automatically cleared after use
* No persistent storage

### 2. Regex Engine

* Fast pattern-based detection
* Handles emails, phone numbers, IDs

### 3. ML Detection Layer

* Context-aware entity detection
* Handles names, locations, sensitive phrases

### 4. Masking Engine

* Replaces detected PII with placeholders
* Example: `john@gmail.com → [EMAIL]`

---

## 🔄 Workflow Example

**Input:**

```
Hi, my name is Raj and my email is raj@gmail.com
```

**Output:**

```
Hi, my name is [NAME] and my email is [EMAIL]
```

---

## 🛠️ Setup

### Clone the repository

```
git clone https://github.com/rajwardhan-patil/shadowprompt.git
cd shadowprompt
```

### Rust Setup

```
cargo build
cargo run
```

### Python Setup (ML Layer)

```
pip install -r requirements.txt
```

---

## 📅 Development Plan

| Day | Task                     |
| --- | ------------------------ |
| 1   | Rust setup + environment |
| 2   | STM implementation       |
| 3   | Regex engine             |
| 4   | ML model training        |
| 5   | API integration          |
| 6   | Testing + optimization   |
| 7   | iOS setup+ Github Actions|
---

## ⚠️ Challenges

* Accurate PII detection across contexts
* Rust ↔ Python integration
* Performance optimization for real-time use
* Mobile deployment constraints

---

## 🔮 Future Scope

* Browser extension for automatic masking
* Mobile app integration
* Enterprise privacy gateway
* Customizable detection rules

---

## 🤝 Contribution

Contributions are welcome. Focus areas:

* Improving detection accuracy
* Expanding regex patterns
* Optimizing performance

---

## 📄 License

MIT License

---

## 👨‍💻 Authors

* Rajwardhan Patil — Systems / Security (Rust)
* Daksh Jain — AI / ML

---

## 🧠 Vision

To make AI **privacy-aware by default**, not an afterthought.


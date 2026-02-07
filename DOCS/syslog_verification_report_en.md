# Syslog Verification Final Report: Encoding Behavior in Real-World Tools

## 1. Purpose of Verification
To demonstrate the discrepancy between the RFC 5424 specification (BOM required for UTF-8) and the actual behavior of real-world Syslog sending utilities, and to confirm the effectiveness of the implemented "Tolerant Parsing."

## 2. Verification Environment
- **Server**: `vlt-syslog-portable` (Mac 192.168.1.22)
- **Client 1 (Mac)**: `nc` (netcat), `logger`
- **Client 2 (Windows)**: `LLM-SVR1` (PowerShell .NET)

## 3. Verification Results: "Raw Data" Analysis from Actual Machines

Analysis of received packets (HEX) recorded in `logs/debug_raw.log`.

| Sending Tool / Format | Expected Behavior (RFC) | Actual Behavior (RAW) | Result | Remarks |
| :--- | :--- | :--- | :--- | :--- |
| **Mac `nc` (UTF-8)** | With BOM | **No BOM** | ✅ UTF-8 (Implicit) | Forgetting BOM is common even in manual transmission |
| **Windows PowerShell** | With BOM | **No BOM** | ✅ UTF-8 (Implicit) | Confirmed BOM omission due to standard .NET behavior |
| **MegaLog / Various SD** | As specified | As specified | ✅ Various Charset | SD `charset` parameter has high reliability |
| **Legacy (RFC 3164)** | Not specified | No BOM (SJIS) | ✅ Shift_JIS (Guess) | Correctly recovered SJIS via statistical estimation |

## 4. Insights and Solutions Regarding the "BOM Trap"
This verification clearly showed that **"Parsing that relies solely on the presence of a BOM does not work in the real world."**

### The Reality
- In standard Windows transmissions (PowerShell, etc.), it is common to omitted the BOM even for UTF-8.
- If we discard this as "No BOM, therefore it's SJIS (or invalid data)," logs from modern systems will all be garbled.

### Our Solution (Three-Tiered Logic)
1. **BOM Detection**: If present, treat as 100% UTF-8.
2. **SD (Structured Data) Parsing**: Analyze `charset` tags (`UTF-8`, `MSG-UTF8`, `Shift_JIS`, etc.) for recovery.
3. **Statistical Estimation (Final Stand)**: Use `chardetng` to make the best judgment for CJK environments based on byte sequence characteristics.

## 5. Conclusion
It has been demonstrated that `vlt-syslog-portable` has extremely high resilience against "imperfect data" in the real world while respecting RFC ideals. This enables stable log monitoring without character corruption even in mixed multi-platform environments.

---

> [!IMPORTANT]
> **A Request to Our Global Users**
> The developer currently only has a **Japanese language environment** for testing. While we have confirmed support for major encodings, we would be very grateful for your reports if you encounter any display issues or "garbled text" in your specific region (English, Korean, Chinese, Vietnamese, etc.). Your feedback helps us make this tool better for everyone!

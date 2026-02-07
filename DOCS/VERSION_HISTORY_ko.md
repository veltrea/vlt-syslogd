# vlt-syslogd 버젼 히스토리

## v0.2.0 (2026-02-07) - "The Real World Provider"
Mac의 `logger`, `nc` 및 Windows의 PowerShell (.NET), MegaLog 등 주요 툴에서 전송되는 Syslog 메시지의 바이너리 분석 결과를 바탕으로 파싱 로직을 대폭 개선했습니다. 대부분의 전송 툴이 RFC 5424(특히 BOM 요구 사항)를 엄격하게 준수하지 않는 실태를 확인했기 때문에, 실제 데이터 구조에 맞춘 "관용적 파싱(Tolerant Parsing)"으로 최적화했습니다.

### [Added / Improved]
- **관용적 파싱 구현**: RFC 5424(신규 규격)에서 BOM이 없는 UTF-8 패킷을 "BOM Trap"으로 인식하고 자동으로 구제하는 로직을 도입했습니다.
- **3단계 디코딩 플로우**: 
  1. BOM (Byte Order Mark) 판정
  2. Structured Data (SD)의 `charset` 파라미터 분석
  3. 통계적 추정(`chardetng`)에 의한 인코딩 추측
- **멀티 플랫폼 검증**: Mac(`logger`, `nc`) 및 Windows(PowerShell, .NET)로부터의 전송 테스트를 완료하고 글자 깨짐 제로를 달성했습니다.
- **기술 문서 정비**: `DOCS/syslog_verification_report.md` 추가. 실지 검증 증거를 기록했습니다.

### [Fixed]
- Windows 등 레거시 소스에서 전송되는 Shift_JIS 메시지가 UTF-8로 오판되는 문제를 수정했습니다.

---

## v0.1.0 (2026-02-01) - "The First Signal"
프로젝트 초기 릴리스. 포터블 버전의 기본 기능 구현.

### [Initial Features]
- Rust를 이용한 고성능 비동기 UDP Syslog 서버 엔진 구축.
- GUI(포터블 버전)를 통한 실시간 로그 모니터링.
- RFC 5424 / RFC 3164의 기본적인 파싱 대응.
- 실용적이고 가벼운 실행 바이너리(포터블 버전) 제공.

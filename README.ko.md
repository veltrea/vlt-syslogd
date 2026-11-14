# Veltrea Syslog Server

[English](README.md) · [日本語](README.ja.md) · 한국어 · [简体中文](README.zh.md) · [繁體中文](README.tw.md) · [Tiếng Việt](README.vi.md)

**메시지마다 Shift_JIS와 UTF-8을 따로 판정해서 디코딩합니다. 한 포트에 인코딩이 섞여 있어도 글자가 깨지지 않습니다.**

**[웹사이트 (English)](https://veltrea.github.io/vlt-syslogd/index.en.html)** · [日本語](https://veltrea.github.io/vlt-syslogd/) · **[릴리스 v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0)**

![Enc 열에 UTF-8, Shift_JIS, Shift_JIS (MSG-SD/BOM-Missing)가 함께 보이는 Portable 로그 화면](DOCS/assets/portable-main.png)

*웹사이트와 같은 화면입니다. v0.4.0 Windows 빌드입니다. Enc 열은 그 행의 판정입니다. `UTF-8`, `Shift_JIS`, 그리고 RFC 5424 구조화 데이터가 문자셋을 선언했을 때의 `Shift_JIS (MSG-SD/BOM-Missing)`가 나옵니다. 메시지를 오른쪽 클릭하고 **Copy as Hex**를 고르면 원래 바이트를 16진수로 복사합니다.*

최신 릴리스는 **v0.4.0**이며 Windows, macOS, Linux(x86_64)용입니다. 어떤 파일을 받을지 모르겠다면 **Portable**부터 시작하세요.

창의 문구는 일본어입니다. Windows와 Linux 메뉴는 `ファイル(File)`, `編集(Edit)`, `ヘルプ(Help)`처럼 병기합니다.

## 세 가지 형태

syslog를 읽는 방식은 같고, 실행 방법만 다릅니다.

| | Portable | Server | Console |
|---|---|---|---|
| 바이너리 | `vlt-syslogd-portable` | `vlt-syslogd-srv` | `vlt-syslogd-console` |
| 역할 | UDP 소켓을 직접 여는 GUI. 설치도 서비스 등록도 필요 없습니다. | 화면이 없는 엔진. Windows 서비스, launchd LaunchDaemon, systemd 중 하나로 동작합니다. 부팅부터 받아 디스크에 씁니다. | 실행 중인 Server에 TCP로 붙는 GUI. 창을 닫아도 수집은 계속됩니다. 서비스 시작, 중지, 재시작도 이 창에 있습니다. |
| 포트 | UDP. 기본값 `0.0.0.0:514` | UDP `514`(syslog). TCP `5141`(Console로의 스트림)과 TCP `5142`(제어)는 기본적으로 `127.0.0.1` | Server의 스트림 주소와 제어 주소에 연결 |
| v0.4.0 배포본 | Windows, macOS, Linux x86_64 | Windows, macOS. Linux는 소스 빌드(`Server/install-linux.sh`는 저장소에 있음. 이 릴리스에는 Linux Server 바이너리가 없음) | Windows, macOS. Linux는 소스 빌드 |

**App**과 **Portable**은 같은 GUI입니다. 데이터를 두는 곳만 다릅니다.

| 형태 | 데이터 디렉터리 |
|---|---|
| **App** | Windows `%APPDATA%\vlt-syslogd`, macOS `~/Library/Application Support/vlt-syslogd`, Linux `$XDG_DATA_HOME/vlt-syslogd` 또는 `~/.local/share/vlt-syslogd` |
| **Portable** | 실행 파일 옆. macOS에서는 `.app` 옆(USB에 넣고 다님. 시스템 안에는 쓰지 않음) |

어느 쪽이든 `VLT_SYSLOGD_DATA_DIR`가 있으면 그 디렉터리를 씁니다. Server의 기본 위치는 시스템 디렉터리입니다. `C:\ProgramData\vlt-syslogd`, `/usr/local/var/vlt-syslogd`, `/var/lib/vlt-syslogd`.

환경 설정(**ファイル → 環境設定…**)에는 항목이 두 개입니다. 수신 포트(**待ち受けポート**)와 로그 디렉터리(**ログ保存先**). **適用**을 누르면 저장됩니다.

## v0.4.0을 받아서 메시지 하나 보내기

아래 파일은 [v0.4.0 릴리스 자산](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0)입니다. 시험 메시지는 로그 첫 행에 나온 포트로 보내세요. `Listening on 0.0.0.0:514 (UDP)`이면 그 포트가 열려 있습니다.

상주시킬 때나 다른 장비에서 받을 때는 [설치 가이드](DOCS/INSTALL.md)를 보세요.

### Windows

| 형태 | 파일 |
|---|---|
| Portable | [vlt-syslogd-portable-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-portable-windows-v0.4.0.zip) → `vlt-syslogd-portable.exe` |
| App | [vlt-syslogd-app-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-app-windows-v0.4.0.zip) → `vlt-syslogd-windows-x86_64-app-v0.4.0.exe` |
| Server | [vlt-syslogd-srv-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-windows-v0.4.0.zip) → `vlt-syslogd-srv.exe`와 `install-windows.ps1` |
| Console | [vlt-syslogd-console-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-windows-v0.4.0.zip) → `vlt-syslogd-console.exe` |

압축을 풀고 Portable exe를 더블 클릭합니다. SmartScreen이 나오면 **추가 정보**를 누른 다음 **실행**을 선택합니다. UDP 514를 여는 데 관리자 권한은 필요 없습니다.

```powershell
$u = New-Object System.Net.Sockets.UdpClient
$b = [Text.Encoding]::UTF8.GetBytes('<132>auth: 認証に失敗しました')
$u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
```

행이 늘어나면 도착한 것입니다. 이 본문은 BOM 없는 UTF-8이며, 이 프로그램이 흔히 받는 형태입니다.

### macOS

| 형태 | 파일 |
|---|---|
| Portable | [vlt-syslogd-macos-portable-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.dmg) 또는 [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.zip) → `vlt-syslogd-portable.app` |
| App | [vlt-syslogd-macos-app-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.dmg) 또는 [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.zip) → `vlt-syslogd.app` |
| Server | [vlt-syslogd-srv-macos-v0.4.0.tar.gz](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-macos-v0.4.0.tar.gz) → `vlt-syslogd-srv`와 `install-macos.sh` |
| Console | [vlt-syslogd-console-macos-app-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-macos-app-v0.4.0.zip) → `vlt-syslogd-console.app` |

빌드는 ad-hoc 서명이며 공증은 되어 있지 않습니다. 내려받은 앱에는 Gatekeeper 격리 속성이 붙습니다. 한 번 지우거나, 앱을 오른쪽 클릭해서 **열기**를 선택하세요.

```bash
xattr -dr com.apple.quarantine vlt-syslogd-portable.app
```

Portable은 데이터를 `.app` 옆에 둡니다. 격리 속성이 남아 있으면 App Translocation 때문에 앱이 그 폴더를 보지 못합니다. CJK 글꼴은 macOS에 들어 있는 히라기노를 사용합니다.

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

macOS `logger`는 로컬 시스템 로그에 씁니다. 호스트와 UDP 포트를 받지 않습니다. 여기서는 `nc`를 사용하세요. 514가 열렸다고 보기 전에 [포트 514](#포트-514)를 확인하세요.

### Linux (x86_64)

이 릴리스에 들어 있는 것은 GUI뿐입니다. Server와 Console의 미리 빌드된 바이너리는 없습니다. 필요하면 소스에서 빌드합니다([BUILD.md](DOCS/BUILD.md), [INSTALL.linux.md](DOCS/INSTALL.linux.md)).

| 형태 | 파일 |
|---|---|
| Portable | [vlt-syslogd-linux-x86_64-portable-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-portable-v0.4.0) |
| App | [vlt-syslogd-linux-x86_64-app-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-app-v0.4.0) |

동적 링크된 x86_64 ELF입니다. 창을 열려면 디스플레이(X11 또는 Wayland)가 필요합니다. v0.4.0 릴리스 노트는 OpenGL도 요구한다고 적고 있습니다.

```bash
chmod +x vlt-syslogd-linux-x86_64-portable-v0.4.0
./vlt-syslogd-linux-x86_64-portable-v0.4.0
```

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

util-linux `logger`는 UDP로 보낼 수 있습니다. macOS `logger`에는 이 옵션이 없습니다. 아래는 Linux용입니다.

```bash
logger -n 127.0.0.1 -P 514 -d 'hello from logger'
```

일본어를 그리려면 GUI가 찾는 CJK 글꼴이 있어야 합니다. Noto Sans CJK(`/usr/share/fonts` 아래의 `NotoSansCJK-Regular.ttc`) 또는 IPA Gothic(`/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf`)입니다. 없으면 □로 나옵니다. Debian/Ubuntu의 `fonts-noto-cjk`는 Noto를 그중 첫 경로에 둡니다.

## 미리 알아 둘 점

### 포트 514

로그 첫 행이 bind 결과입니다. 실패 행이 있는 동안 514로 보낸 메시지는 화면에 나오지 않습니다.

- **Windows.** UDP 514를 여는 데 관리자는 필요 없습니다. 설치 프로그램이 관리자를 요구하는 때는 서비스를 등록할 때뿐입니다.
- **Linux.** 1024 미만 포트는 root 또는 `CAP_NET_BIND_SERVICE`가 필요합니다. Portable 바이너리라면 다음과 같습니다.

  ```bash
  sudo setcap 'cap_net_bind_service=+ep' ./vlt-syslogd-linux-x86_64-portable-v0.4.0
  ```

  `Server/install-linux.sh`는 root로 동작하는 systemd 유닛을 설치합니다. 유닛 파일에는 `AmbientCapabilities=CAP_NET_BIND_SERVICE`를 쓰는 주석 처리된 예도 있습니다.
- **macOS.** root가 아닌 상태에서 `0.0.0.0:514` bind가 `EACCES`를 반환했다는 기록이 [DOCS/portable_macos_port_behavior.md](DOCS/portable_macos_port_behavior.md)에 있습니다. 앱 안의 안내 문구는 여전히 macOS의 와일드카드 bind에는 root가 필요 없다고 표시합니다. 기준은 로그 첫 행입니다.

bind에 실패하면 배너에 **5514**가 채워집니다. **待ち受け開始**는 그 세션 동안만 그 포트로 받으며 저장하지 않습니다. 남기려면 **環境設定**에서 **適用**을 누르거나 `VLT_SYSLOGD_BIND=0.0.0.0:5514`로 실행하세요. 시험 메시지는 로그 행이 가리키는 포트로 보내세요. macOS에서 514 자체를 잡으려면 관리자 권한으로 Portable을 실행하거나, root LaunchDaemon으로 설치되는 Server(`com.veltrea.vlt-syslogd-srv`)를 사용하세요.

### macOS Gatekeeper

이 빌드 뒤에는 Apple Developer 계정이 없습니다. 오른쪽 클릭 → **열기**를 하거나 위의 `com.apple.quarantine`을 지우기 전에는 첫 실행이 멈춥니다.

### 서비스가 떠 있어도 도달하지 않을 수 있습니다

다른 장비에서 닿으려면 `bind_addr`이 `0.0.0.0` 또는 LAN 주소이고, 방화벽이 UDP 514를 허용해야 합니다. 스트림(`5141`)과 제어(`5142`)는 바꾸지 않는 한 루프백에 있습니다. 이 둘을 기기 밖으로 열면 Server 설정을 다시 쓸 수 있는 `set_config`도 노출됩니다. 다른 머신의 Console을 쓸 때는 설치 가이드가 SSH 터널을 권합니다.

```powershell
# Windows, 관리자 PowerShell
New-NetFirewallRule -DisplayName "vlt-syslogd 514/udp" -Direction Inbound -Protocol UDP -LocalPort 514 -Action Allow
```

```bash
# Linux
sudo ufw allow 514/udp
# firewalld:
sudo firewall-cmd --permanent --add-port=514/udp && sudo firewall-cmd --reload
```

macOS 애플리케이션 방화벽은 앱 단위입니다. 물으면 `vlt-syslogd-srv`의 들어오는 연결을 허용하세요.

## 배경

일본어가 제대로 나오지 않던 가벼운 Windows용 syslog 서버를 오래 쓰다가, Rust로 처음부터 작성했습니다. 데이터그램마다 따로 디코딩합니다. BOM, RFC 5424 구조화 데이터의 `charset`, 또는 바이트 자체(`chardetng`를 이용한 Shift_JIS 추정 포함)입니다. Enc 열이 그 판정입니다. 유지보수는 건강과 기력이 닿는 동안 이어 갑니다. 작성자는 veltrea입니다.

## 문서

| | |
|---|---|
| 설치 | [INSTALL.md](DOCS/INSTALL.md) — [Windows](DOCS/INSTALL.windows.md) · [macOS](DOCS/INSTALL.macos.md) · [Linux](DOCS/INSTALL.linux.md) |
| 포트와 서비스 이름 | [REFERENCE.md](DOCS/REFERENCE.md) |
| 빌드 | [BUILD.md](DOCS/BUILD.md) — `cargo build --release --workspace` (edition 2024) |
| 인코딩 메모 | [syslog_verification_report.ko.md](DOCS/syslog_verification_report.ko.md) |

## 라이선스

[MIT](LICENSE). Copyright (c) 2026 veltrea.

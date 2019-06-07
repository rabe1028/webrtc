# webrtc
Pure Rust WebRTC library

# 参考

quiche - cloudflare

(BSD 2-Clause "Simplified" License)

Rustでのパケット作成部分を参考にした

aiortc

WebRTCのコードの作成を参考にした

pions/webrtc

テストコードが簡潔に纏まっているので，
テストケースを流用した．

# 実装

- [x] RTP
    - [x] RTP Packet Struct
    - [x] RTP Packet Serialize and Deserializer
    - [] add Header Extension (H.264, H.265, VP8, VP9 etc...)
    - [x] Serialize and Deserialize test

- [] RTCP Packet
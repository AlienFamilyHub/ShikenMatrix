import gsap from "gsap";
import { onMount } from "solid-js";
import IconBox from "~icons/mingcute/box-line";
import IconInfo from "~icons/mingcute/information-line";
import IconSend from "~icons/mingcute/send-line";
import IconWifi from "~icons/mingcute/wifi-line";

export function AboutPage() {
  let mainRef!: HTMLElement;

  onMount(() => {
    gsap.from(mainRef.children, {
      opacity: 0,
      y: 6,
      duration: 0.35,
      stagger: 0.08,
      ease: "power2.out",
    });
  });

  return (
    <main class="about-page" ref={mainRef}>
      <section class="about-hero">
        <div class="about-icon">
          <IconInfo />
        </div>
        <div>
          <h2>ShikenMatrix</h2>
          <p>桌面活动监听与上报工具。Desktop 只连接 ShikenMatrix server，由 server 统一转发上游。</p>
        </div>
      </section>

      <section class="about-grid">
        <div class="about-item">
          <IconWifi />
          <div>
            <h3>Server WS</h3>
            <p>WebSocket 长连接实时发送窗口、媒体和封面上传事件到 ShikenMatrix server。</p>
          </div>
        </div>
        <div class="about-item">
          <IconSend />
          <div>
            <h3>服务端上游</h3>
            <p>Native WebSocket、MX-Space 与 S3 配置均由 Server Admin 管理并持久化到 SQLite。</p>
          </div>
        </div>
        <div class="about-item">
          <IconBox />
          <div>
            <h3>项目边界</h3>
            <p>监听可以单独运行；启动上报须依赖已启动的监听。</p>
          </div>
        </div>
      </section>
    </main>
  );
}

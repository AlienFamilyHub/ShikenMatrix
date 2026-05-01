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
          <p>桌面活动监听与上报工具。支持独立监听、Native WebSocket 上报和 Mix-Space HTTP 上报。</p>
        </div>
      </section>

      <section class="about-grid">
        <div class="about-item">
          <IconWifi />
          <div>
            <h3>Native 方案</h3>
            <p>WebSocket 长连接实时发送窗口、媒体和封面上传事件。</p>
          </div>
        </div>
        <div class="about-item">
          <IconSend />
          <div>
            <h3>Mix-Space 方案</h3>
            <p>按 ProcessReporter 的 MixSpace payload 形状发送 HTTP JSON。</p>
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

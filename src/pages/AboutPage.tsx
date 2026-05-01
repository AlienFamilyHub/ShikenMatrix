import { onMount } from 'solid-js'
import gsap from 'gsap'
import IconGithub from '~icons/mingcute/github-line'
import IconInfo from '~icons/mingcute/information-line'
import IconUpload from '~icons/mingcute/upload-2-line'

export function AboutPage() {
  let mainRef!: HTMLElement

  onMount(() => {
    gsap.from(mainRef.children, {
      opacity: 0,
      y: 20,
      duration: 0.6,
      stagger: 0.15,
      ease: 'power3.out',
    })
    
    // Animate the icons inside about items
    gsap.from(mainRef.querySelectorAll('.about-item'), {
      opacity: 0,
      scale: 0.95,
      duration: 0.5,
      stagger: 0.1,
      delay: 0.3,
      ease: 'back.out(1.2)'
    })
  })

  return (
    <main class="about-page" ref={mainRef}>
      <section class="about-hero">
        <div class="about-icon">
          <IconInfo />
        </div>
        <div>
          <h2>ShikenMatrix</h2>
          <p>桌面活动监听与上报工具，支持独立监听、Native WebSocket 上报和 Mix-Space HTTP 上报。</p>
        </div>
      </section>

      <section class="about-grid">
        <div class="about-item">
          <IconUpload />
          <div>
            <h3>Native 方案</h3>
            <p>使用 WebSocket 长连接实时发送窗口、媒体和封面上传事件。</p>
          </div>
        </div>
        <div class="about-item">
          <IconUpload />
          <div>
            <h3>Mix-Space 方案</h3>
            <p>按 ProcessReporter 的 MixSpace payload 形状发送 HTTP JSON。</p>
          </div>
        </div>
        <div class="about-item">
          <IconGithub />
          <div>
            <h3>项目边界</h3>
            <p>监听可以单独运行；启动上报必须依赖已启动的监听。</p>
          </div>
        </div>
      </section>
    </main>
  )
}

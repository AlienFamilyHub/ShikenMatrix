import type { CloseBehavior } from '../types'

interface CloseChoiceModalProps {
  remember: boolean
  onRememberChange: (remember: boolean) => void
  onApply: (behavior: CloseBehavior) => void
}

export function CloseChoiceModal(props: CloseChoiceModalProps) {
  return (
    <div class="modal-backdrop">
      <div class="modal-panel" role="dialog" aria-modal="true" aria-labelledby="close-choice-title">
        <div class="modal-header">
          <h2 id="close-choice-title">关闭窗口时如何处理？</h2>
          <p>选择后，本次关闭会立即执行；勾选后以后会直接按这个选择处理。</p>
        </div>

        <label class="checkbox-label close-choice-remember">
          <input
            type="checkbox"
            checked={props.remember}
            onChange={event => props.onRememberChange(event.currentTarget.checked)}
          />
          <span>记住我的选择</span>
        </label>

        <div class="modal-actions">
          <button class="btn btn-secondary modal-button" onClick={() => props.onApply('quit')}>
            直接退出
          </button>
          <button class="btn btn-primary modal-button" onClick={() => props.onApply('hide_to_tray')}>
            隐藏到托盘
          </button>
        </div>
      </div>
    </div>
  )
}

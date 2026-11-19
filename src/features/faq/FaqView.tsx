import { Icon } from "../../components/Icon";

export interface FaqItem {
  question: string;
  answer: string;
}

interface FaqViewProps {
  faqItems: FaqItem[];
  openFaq: number | null;
  onToggleFaq: (index: number) => void;
}

export function FaqView({ faqItems, openFaq, onToggleFaq }: FaqViewProps) {
  return (
    <div className="view-container">
      <div className="view-header">
        <div className="view-header-copy">
          <h1>Preguntas Frecuentes y Ayuda</h1>
          <p>
            Todo lo que necesitas saber sobre el uso y las herramientas locales integradas en FileFlow Desktop.
          </p>
        </div>
      </div>

      <div className="faq-list">
        {faqItems.map((item, index) => {
          const isOpen = openFaq === index;
          return (
            <div key={item.question} className={isOpen ? "faq-item open" : "faq-item"}>
              <button
                type="button"
                onClick={() => onToggleFaq(index)}
                aria-expanded={isOpen}
              >
                <span>{item.question}</span>
                <Icon name="arrow" size={14} />
              </button>
              {isOpen && <p>{item.answer}</p>}
            </div>
          );
        })}
      </div>
    </div>
  );
}

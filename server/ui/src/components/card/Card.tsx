import { children, ParentComponent } from 'solid-js';
import styles from './Card.module.css';

const Card: ParentComponent = (props) => {
  const c = children(() => props.children);

  return (
    <div class={styles.Card}>
      {c()}
    </div>
  );
};

export default Card;
import type { Component } from "solid-js";
import styles from "./App.module.css";
import Card from "./components/card/Card";
import HueCard from "./components/hue/hue-card/HueCard";

const App: Component = () => {
  return (
    <div class={styles.App}>
      <header class={styles.header}></header>
      <div class={styles.gridWrapper}>
        <div class={styles.one}>
          <Card></Card>
        </div>
        <div class={styles.two}>
          <Card>
            <HueCard></HueCard>
          </Card>
        </div>
        <div class={styles.three}>
          <Card></Card>
        </div>
        <div class={styles.four}>
          <Card></Card>
        </div>
      </div>
    </div>
  );
};

export default App;

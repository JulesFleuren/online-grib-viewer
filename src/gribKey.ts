class GribKey {
  isVectorField: boolean;
  firstComponent: string;
  secondComponent: string | null;

  constructor(stringKey: string) {
    if (stringKey.startsWith("vector")) {
      this.isVectorField = true;
      let components = stringKey.split(":");
      if (components.length < 2) {
        throw new Error("Invalid vector field key format");
      }
      components = components[1].split(",");
      if (components.length < 2) {
        throw new Error("Invalid vector field key format");
      }
      this.firstComponent = components[0];
      this.secondComponent = components[1];
    } else {
      this.isVectorField = false;
      this.firstComponent = stringKey;
      this.secondComponent = null;
    }
  }

  toString(): string {
    if (this.isVectorField) {
      return `vector:${this.firstComponent},${this.secondComponent}`;
    } else {
      return this.firstComponent;
    }
  }
}

export { GribKey };

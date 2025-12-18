export interface Ingredient {
  id: string;
  name?: string;
  code?: string;
  amount: number;
  percentage: number;
  unitCost: number;
  cost: number;
  costPercentage: number;
}

export interface Nutrient {
  id: string;
  name?: string;
  code?: string;
  value: number;
  unit?: string;
}

export interface ShadowPrice {
  constraint: string;
  value: number;
  interpretation: string;
}

export interface Violation {
  constraint: string;
  required: number;
  actual: number;
  violationAmount: number;
  description: string;
}

export interface SolveResult {
  status: "optimal" | "infeasible" | "error";
  formula: string;
  formulaName?: string;
  formulaCode?: string;
  description?: string;
  batchSize: number;
  totalCost: number;
  ingredients: Ingredient[];
  nutrients: Nutrient[];
  analysis?: {
    bindingConstraints: string[];
    shadowPrices: ShadowPrice[];
  };
  violations: Violation[];
}

export interface MenuItem {
  id: string;
  name: string;
  description: string;
  price_zec: number;
  image: string;
}

export const MENU_ITEMS: MenuItem[] = [
  {
    id: 'sunny-side-up',
    name: 'Sunny Side Up',
    description: 'Fresh eggs cooked to perfection with crispy edges',
    price_zec: 0.0002,
    image: '/menu/sunny-side-up.svg',
  },
  {
    id: 'latte',
    name: 'Cafe Latte',
    description: 'Smooth espresso with velvety steamed milk',
    price_zec: 0.0001,
    image: '/menu/latte.svg',
  },
  {
    id: 'croissant',
    name: 'Butter Croissant',
    description: 'Flaky, buttery French pastry baked fresh daily',
    price_zec: 0.00015,
    image: '/menu/croissant.svg',
  },
];

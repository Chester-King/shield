import { Navbar } from '@/components/layout/Navbar';
import { Footer } from '@/components/layout/Footer';
import { Hero } from '@/components/sections/Hero';
import { Problem } from '@/components/sections/Problem';
import { HowItWorks } from '@/components/sections/HowItWorks';
import { Features } from '@/components/sections/Features';
import { Technical } from '@/components/sections/Technical';
import { Security } from '@/components/sections/Security';
import { CTA } from '@/components/sections/CTA';

export default function Home() {
  return (
    <>
      <Navbar />
      <main className="">
        <Hero />
        <Problem />
        <HowItWorks />
        <Features />
        <Technical />
        <Security />
        <CTA />
      </main>
      <Footer />
    </>
  );
}

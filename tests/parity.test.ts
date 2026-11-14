import {describe, expect, test} from "bun:test";
import {resolve} from "node:path";
import * as framing from "../crates/prism/src/framing.js";
import * as optics from "../crates/prism/src/optics.js";
import * as scene from "../crates/prism/src/types.js";
import * as interaction from "../crates/prism/src/interaction.js";

// Reference code is test-only: the application never imports upstream TS.
const upstream = resolve(process.env.VGPU_UPSTREAM_DIR ?? "../vgpu");
const revision = Bun.spawnSync(["git", "rev-parse", "HEAD"], {cwd: upstream, timeout: 10_000});
expect(revision.exitCode).toBe(0);
expect(revision.stdout.toString().trim()).toBe("773351f8c96d1cbec28155c0e36ae3efd1cba135");
const original = resolve(upstream, "apps/docs/app/[lang]/(home)/components/prism-background");
const referenceFraming = await import(original + "/scene/framing.ts");
const referenceInteraction = await import(original + "/runtime/interaction.ts");
const referenceOptics=await import(original+"/scene/optics.ts");
const referenceTypes=await import(original+"/types.ts");

function close(actual: unknown, expected: unknown) {
  if (typeof actual === "number" && typeof expected === "number") {
    if (!Number.isFinite(expected)) expect(Object.is(actual, expected)).toBe(true);
    else expect(actual).toBeCloseTo(expected, 12);
  } else if (Array.isArray(expected)) {
    expect(Array.isArray(actual)).toBe(true);
    expect((actual as unknown[]).length).toBe(expected.length);
    expected.forEach((value, i) => close((actual as unknown[])[i], value));
  } else if (expected && typeof expected === "object") {
    expect(Object.keys(actual as object).sort()).toEqual(Object.keys(expected).sort());
    for (const [key, value] of Object.entries(expected)) close((actual as Record<string, unknown>)[key], value);
  } else expect(actual).toEqual(expected);
}

describe("compiled Rust against pinned upstream", () => {
  test("mobile pointer and easing match through movement, settling, and leave", () => {
    const port = interaction.prismInteractionDefault.default();
    let invalidations = 0;
    const canvas = {getBoundingClientRect: () => ({left: 20, top: 40, width: 400, height: 300})};
    const source = referenceInteraction.createPrismInteraction(canvas, () => invalidations++);
    for (const time of [0, 4.5, 9, 18, -3, 112.1, NaN, Infinity, -Infinity]) {
      close(interaction.automaticPointerPosition(time), referenceInteraction.automaticPointerPosition(time));
    }
    for (const position of [[0.84, 0.5], [1, 0], [-2, 4], [0.3, 0.7]]) {
      interaction.PrismInteraction.setNormalizedPointer(port, position);
      source.setNormalizedPointer(position);
      for (let frame = 0; frame < 200; frame++) {
        close(interaction.PrismInteraction.stepAim(port), source.stepAim());
        close(interaction.PrismInteraction.stepOrbit(port), source.stepOrbit());
      }
    }
    expect(invalidations).toBe(0);
    const moved = interaction.PrismInteraction.onPointerMove(port, true, [160, 60], [20, 40], [400, 300]);
    source.onPointerMove({isPrimary: true, clientX: 160, clientY: 60});
    expect(moved).toBe(true);
    expect(invalidations).toBe(1);
    close(interaction.PrismInteraction.stepAim(port), source.stepAim());
    close(interaction.PrismInteraction.stepOrbit(port), source.stepOrbit());
    interaction.PrismInteraction.onPointerLeave(port);
    source.onPointerLeave();
    for (let frame = 0; frame < 200; frame++) {
      close(interaction.PrismInteraction.stepAim(port), source.stepAim());
      close(interaction.PrismInteraction.stepOrbit(port), source.stepOrbit());
    }
  });

  test("viewport clipping matches including empty and invalid dimensions", () => {
    const canvases = [
      {left:200, top:80, width:1000, height:500},
      {left:100, top:50, width:400, height:200},
      {left:0, top:0, width:0, height:200},
      {left:0, top:0, width:NaN, height:200},
    ];
    const targets = [
      {left:700, top:130, width:450, height:400},
      {left:0, top:0, width:300, height:400},
      {left:600, top:50, width:100, height:100},
      {left:0, top:0, width:NaN, height:100},
    ];
    for (const canvas of canvases) for (const target of targets) {
      close(framing.viewportWithinCanvas(canvas,target), referenceFraming.viewportWithinCanvas(canvas,target));
    }
  });

  test("projection values, float32 rounding and identity match", () => {
    const matrix = new Float32Array([1.2,0.3,0.1,0.2,0,1.7,0,0,0,0,1,0,0.6,-0.1,0,1]);
    for (const target of [
      {scale:1,offset:[0,0]},
      {scale:0.5,offset:[0.25,-0.2]},
      {scale:1.7,offset:[-0.37,0.19]},
    ]) {
      const input = Array.from(matrix);
      const result = framing.applyProjectionFraming(input,target);
      const reference = referenceFraming.applyProjectionFraming(matrix,target);
      close(result._0,Array.from(reference));
      expect(result._0 === input).toBe(reference === matrix);
      expect(input).toEqual(Array.from(matrix));
      close(framing.framingCoverage(target),referenceFraming.framingCoverage(target));
    }
    for (const points of [[], [[0,0,0]], [[-0.6,-0.2,1],[0.4,0.8,-1]], [[NaN,0,0]]]) {
      close(framing.projectedBounds([Array.from(matrix)],points),referenceFraming.projectedBounds([matrix],points));
    }
    close(framing.projectedBounds([],[]),referenceFraming.projectedBounds([],[]));
  });

  test("camera-fit binary search and silhouette alignment match", () => {
    for (const viewport of [
      {left:0.5,top:0,right:1,bottom:1},
      {left:0,top:0,right:1,bottom:1},
      {left:0.3,top:72/836,right:1416/1440,bottom:764/836},
    ]) for (const span of [1,10,100]) {
      const bounds = (distance:number) => ({x0:-span/distance,y0:-0.5/distance,x1:span/distance,y1:0.5/distance});
      const result=framing.fitProjectionDistance(viewport,bounds,0.5,10,36);
      close(result,referenceFraming.fitProjectionDistance(viewport,bounds,0.5,10,36));
      close(framing.alignProjection(result.bounds,viewport),referenceFraming.alignProjection(result.bounds,viewport));
    }
  });
});

describe("compiled Rust prism optics against pinned upstream", () => {
  test("geometric predicates, intersections, Snell and Fresnel match", () => {
    const triangles = [referenceTypes.PRISM_TRIANGLE, {a:[0,1],b:[-1,-1],c:[1,-1]}];
    for (const triangle of triangles) {
      close(optics.triangleWinding(triangle),referenceOptics.triangleWinding(triangle));
      for (const point of [[0,0],[1,1],[-0.1,0.3],[0,-1]]) {
        expect(optics.insideTriangle(triangle,point)).toBe(referenceOptics.insideTriangle(triangle,point));
      }
      for (const origin of [[2,0],[-2,0],[0,0],[0,2]]) for (const direction of [[-1,0],[1,0],[0,-1]]) {
        close(optics.intersectTriangle(triangle,origin,direction,1e-4),referenceOptics.intersectTriangle(triangle,origin,direction,1e-4));
      }
    }
    for (let degrees=0;degrees<=90;degrees+=3) for (const eta of [1/1.5,1,1.5,1.8]) {
      const angle=degrees*Math.PI/180;
      const incident=[Math.sin(angle),-Math.cos(angle)];
      const normal=[0,1];
      close(optics.refract(incident,normal,eta),referenceOptics.refract(incident,normal,eta));
      close(optics.reflect(incident,normal),referenceOptics.reflect(incident,normal));
      close(optics.fresnelTransmittance(incident,normal,eta,1),referenceOptics.fresnelTransmittance(incident,normal,eta,1));
    }
  });

  test("whole ray paths match across incidence, wavelength, and bounce budgets", () => {
    for (const incidence of [-20,0,30,44,48,60,75,89,120]) {
      const lamp=referenceTypes.lampForIncidence(incidence);
      for (const wavelength of [400,450,550,650,700]) {
        const ior=optics.iorAt(wavelength,1.3,0.01);
        close(ior,referenceOptics.iorAt(wavelength,1.3,0.01));
        for (const budget of [0,1,3,6]) {
          close(optics.tracePrism(referenceTypes.PRISM_TRIANGLE,lamp.center,lamp.direction,ior,budget),
            referenceOptics.tracePrism(referenceTypes.PRISM_TRIANGLE,lamp.center,lamp.direction,ior,budget));
          close(optics.tracePrismDetailed(referenceTypes.PRISM_TRIANGLE,lamp.center,lamp.direction,ior,budget),
            referenceOptics.tracePrismDetailed(referenceTypes.PRISM_TRIANGLE,lamp.center,lamp.direction,ior,budget));
        }
      }
    }
  });

  test("CIE and D65 spectral colors match throughout and outside the visible range", () => {
    for (const wavelength of [...Array.from({length: 361},(_,i)=>370+i),NaN,Infinity,-Infinity]) {
      close(optics.wavelengthToLinearRgb(wavelength),referenceOptics.wavelengthToLinearRgb(wavelength));
      close(optics.wavelengthToBeamRgb(wavelength),referenceOptics.wavelengthToBeamRgb(wavelength));
    }
  });
});

describe("compiled Rust scene definition against pinned upstream", () => {
  test("beam width and lamp geometry match at all tested incidences", () => {
    for(const width of [-1,0,0.01,0.025,0.1,0.2,1,NaN,Infinity,-Infinity]) {
      close(scene.clampBeamWidth(width),referenceTypes.clampBeamWidth(width));
      for(const incidence of [-85,-35,0,30,60,75,85])for(const position of [-1,0,0.5,1,2]) {
        close(scene.lampForIncidence(incidence,width,position),referenceTypes.lampForIncidence(incidence,width,position));
      }
    }
    for(const position of [-1,0,0.3,0.5,1,2,NaN]) {
      close(scene.prismEntryPoint(position),referenceTypes.prismEntryPoint(position));
    }
  });

  test("collimated-light validation preserves the original error message", () => {
    const cases=[[[0,0],[0,0]],[[NaN,0],[1,0]],[[0,0],[Infinity,1]],[[0,0],[1e-10,0]]];
    for(const [center,target] of cases) {
      let message="";
      try { referenceTypes.collimatedLightBetween(center,target,0.025); } catch(error) { message=(error as Error).message; }
      expect(message).not.toBe("");
      expect(()=>scene.collimatedLightBetween(center,target,0.025)).toThrow(message);
    }
    close(scene.collimatedLightBetween([1,2],[4,6],0.025),referenceTypes.collimatedLightBetween([1,2],[4,6],0.025));
  });
});

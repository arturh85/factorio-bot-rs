import { shallowMount } from "@vue/test-utils";
import Player from "@/components/Player.vue";

describe("Player.vue", () => {
  it("renders props.msg when passed", () => {
    const msg = "new message";
    const wrapper = shallowMount(Player, {
      propsData: { msg },
    });
    expect(wrapper.text()).toMatch(msg);
  });
});

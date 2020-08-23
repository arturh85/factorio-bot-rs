import {State} from "@/store";
import {Store} from "vuex";

import type {FactorioEntity, World,} from "@/factorio-bot/types";
import {Direction, Entities, InventoryType, Technologies} from "@/factorio-bot/types";
import {emptyWorld} from "@/factorio-bot/util";
import {FactorioApi} from "@/factorio-bot/restApi";
import {blueprintTileableStarterSteamEngineBoiler} from "@/factorio-bot/blueprints";
import {FactorioBot} from "@/factorio-bot/bot";
import {executeTask, Task, TaskStatus} from "@/factorio-bot/task";
import {createResearchTask} from "@/factorio-bot/tasks/research-task";
import {createBuildStarterMinerFurnaceTask} from "@/factorio-bot/tasks/build-starter-miner-furnace-task";
import {createBuildStarterMinerCoalTask} from "@/factorio-bot/tasks/build-starter-miner-coal-task";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";
import {createBuildStarterBase} from "@/factorio-bot/tasks/build-starter-base-task";
import {createBuildStarterMinerChestTask} from "@/factorio-bot/tasks/build-starter-miner-chest-task";

const STORAGE_KEY = "world";

// technologies we want to research first
const PRIORITY_RESEARCH = ["automation"];


export class FactorioBotManager {
    $store: Store<State>;
    bots: FactorioBot[] = [];

    constructor(store: Store<State>) {
        this.$store = store
    }

    async init(): Promise<FactorioBot[]> {
        const players = await FactorioApi.allPlayers();
        if (players.length === 0) {
            throw new Error("no players logged in!");
        }
        console.log("available bots:");
        for (const player of players) {
            console.log(player);
        }
        const recipes = await FactorioApi.allRecipes();
        const force = await FactorioApi.playerForce();
        const entityPrototypes = await FactorioApi.allEntityPrototypes();
        const itemPrototypes = await FactorioApi.allItemPrototypes();
        const stored = await FactorioApi.retrieveMapData<World>("world");

        this.$store.commit('updateEntityPrototypes', entityPrototypes)
        this.$store.commit('updateItemPrototypes', itemPrototypes)
        this.$store.commit('updatePlayers', players)
        this.$store.commit('updateForce', force)
        this.$store.commit('updateRecipes', recipes)
        const world = stored ? stored : {...emptyWorld};
        this.$store.commit('updateWorld', world)
        if (stored) {
            console.log("using existing world", world);
        }
        this.bots = players.map(
            (player) => new FactorioBot(this.$store, player.playerId)
        );
        return this.bots;
    }

    findAvailableTasks(): Task[] {
        const available: Task[] = [];
        for (const task of this.$store.state.tasks) {
            if (task.status === TaskStatus.NEW || task.status === TaskStatus.STARTED) {
                available.push(task);
            }
        }
        return available;
    }

    public async processTasks(): Promise<void> {
        // console.log("*** processing tasks", this.$store.state.tasks);
        let doneSomething = false
        // while (true) {
        const availableTasks = this.$store.state.tasks.filter(task =>
            task.status === TaskStatus.NEW
            || task.status === TaskStatus.WAITING)
        if (availableTasks.length === 0) {
            console.log("no available tasks remain, stopping");
            // break;
            return;
        }
        const availableBots = this.bots.filter(bot => bot.busyWith === null)
        // console.log('bots:', this.bots)
        // console.log('available bots:', availableBots)
        for (let i = 0; i < availableTasks.length && i < this.bots.length; i++) {
            try {
                await executeTask(this.$store, availableBots, availableTasks[i])
            } catch(err) {}
            doneSomething = true
        }
        // }
        // console.log("*** processing tasks finished");
        if (doneSomething) {
            await this.saveWorldInMap()
        }
    }

    async testBuildIronMinerSmelter(n: number): Promise<void> {
        const task = await createBuildStarterMinerFurnaceTask(this.$store, Entities.ironOre, Entities.ironPlate, n)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async testBuildCopperMinerSmelter(n: number): Promise<void> {
        const task = await createBuildStarterMinerFurnaceTask(this.$store, Entities.copperOre, Entities.copperPlate, n)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async testBuildStoneMinerChest(n: number): Promise<void> {
        const task = await createBuildStarterMinerChestTask(this.$store, Entities.stone, n)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async testPlaceOffshorePump(): Promise<void> {
        this.bots[0].cheatItem(Entities.offshorePump, 1)
        this.bots[0].placeOffshorePump()
    }

    async testBuildCoalLoop(n: number): Promise<void> {
        const task = await createBuildStarterMinerCoalTask(this.$store, n)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async testCraftTask(name: string, count: number): Promise<void> {
        const task = await createCraftTask(this.$store, name, count, false)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async testGatherTask(name: string, count: number): Promise<void> {
        const task = await createGatherTask(this.$store, name, count)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async buildStarterBase(): Promise<void> {
        const task = await createBuildStarterBase(this.$store, 6, 4, 2, 4, 2)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async researchRocketsilo(): Promise<void> {
        const task = await createResearchTask(this.$store, Technologies.rocketSilo)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async researchAutomation(): Promise<void> {
        const task = await createResearchTask(this.$store, Technologies.automation)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async researchLogistics(): Promise<void> {
        const task = await createResearchTask(this.$store, Technologies.logistics)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async researchSteelProcessing(): Promise<void> {
        const task = await createResearchTask(this.$store, Technologies.steelProcessing)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async researchLogisticSciencePack(): Promise<void> {
        const task = await createResearchTask(this.$store, Technologies.logisticSciencePack)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async updatePlayers(): Promise<void> {
        this.$store.commit('updatePlayers', await FactorioApi.allPlayers())
    }
    async saveWorldAndServer(): Promise<void> {
        await this.saveWorldInMap()
        await FactorioApi.saveServer()
    }

    async testCheatStuff(): Promise<void> {
        const bot = this.bots[0];
        await bot.cheatItem(Entities.burnerMiningDrill, 50);
        await bot.cheatItem(Entities.stoneFurnace, 50);
        await bot.cheatItem(Entities.offshorePump, 5);
        await bot.cheatItem(Entities.steamEngine, 5);
        await bot.cheatItem(Entities.boiler, 2);
        await bot.cheatItem(Entities.smallElectricPole, 10);
        await bot.cheatItem(Entities.pipe, 50);
        await bot.cheatItem(Entities.pipeToGround, 50);
        await bot.cheatItem(Entities.transportBelt, 50);
        await bot.cheatItem(Entities.ironChest, 10);
    }

    async saveWorldInMap(): Promise<void> {
        await FactorioApi.storeMapData(STORAGE_KEY, this.$store.state.world);
    }
}

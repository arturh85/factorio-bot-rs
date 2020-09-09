import {State} from "@/store";
import {Store} from "vuex";

import type {World,} from "@/factorio-bot/types";
import {Direction, Entities, EntityTypes, Position, Technologies} from "@/factorio-bot/types";
import {emptyWorld} from "@/factorio-bot/util";
import {FactorioApi} from "@/factorio-bot/restApi";
import {FactorioBot} from "@/factorio-bot/bot";
import {availableBots, executeTask, Task, TaskStatus} from "@/factorio-bot/task";
import {createResearchTask} from "@/factorio-bot/tasks/research-task";
import {createBuildStarterMinerFurnaceTask} from "@/factorio-bot/tasks/build-starter-miner-furnace-task";
import {createBuildStarterMinerCoalTask} from "@/factorio-bot/tasks/build-starter-miner-coal-task";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";
import {createBuildStarterBase} from "@/factorio-bot/tasks/build-starter-base-task";
import {createBuildStarterMinerChestTask} from "@/factorio-bot/tasks/build-starter-miner-chest-task";
import {createStartRocketTask} from "@/factorio-bot/tasks/start-rocket-task";
import {createBuildMinerLineTask} from "@/factorio-bot/tasks/build-miner-line-task";
import {createBuildFurnaceLineTask} from "@/factorio-bot/tasks/build-furnace-line-task";

const STORAGE_KEY = "world";

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
        // console.log('bots:', this.bots)
        // console.log('available bots:', availableBots)
        for (const task of availableTasks) {
            try {
                await executeTask(this.$store, await availableBots(this.$store), task)
            } catch (err) {
            }
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

    // async testPlaceOffshorePump(): Promise<void> {
    //     await this.bots[0].cheatItem(Entities.offshorePump, 1)
    //     await this.bots[0].placeOffshorePump()
    // }

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

    async testBuildMinerLine(oreName: string): Promise<void> {
        const task = await createBuildMinerLineTask(this.$store, oreName)
        this.$store.commit('pushTask', task)
        await this.processTasks();
    }

    async testBuildFurnaceLine(oreName: string, plateName: string): Promise<void> {
        const task = await createBuildFurnaceLineTask(this.$store, oreName, plateName)
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


    async startRocket(): Promise<void> {
        const task = await createStartRocketTask(this.$store)
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

    async buildBeltPath( fromPosition: Position,
                         toPosition: Position,
                         toDirection: Direction): Promise<void> {
        await FactorioApi.cheatTechnology(Technologies.automation);
        await FactorioApi.cheatTechnology(Technologies.logistics);
        const entities = await FactorioApi.planPath(Entities.transportBelt, EntityTypes.transportBelt, Entities.undergroundBelt, EntityTypes.undergroundBelt, 5, fromPosition, toPosition, toDirection);
        const bot = this.bots[0]
        const cnt = entities.length + 5;
        await bot.cheatItem(Entities.transportBelt, cnt);
        await bot.cheatItem(Entities.undergroundBelt, cnt);
        for (const entity of entities) {
            await bot.placeEntity(entity.name, entity.position, entity.direction)
        }
    }

    async buildPipePath( fromPosition: Position,
                         toPosition: Position,
                         toDirection: Direction): Promise<void> {
        await FactorioApi.cheatTechnology(Technologies.automation);
        await FactorioApi.cheatTechnology(Technologies.logistics);
        const entities = await FactorioApi.planPath(Entities.pipe, EntityTypes.pipe, Entities.pipeToGround, EntityTypes.pipeToGround, 5, fromPosition, toPosition, toDirection);
        const bot = this.bots[0]
        await bot.cheatItem(Entities.pipe, entities.length + 5);
        await bot.cheatItem(Entities.pipeToGround, entities.length + 5);
        for (const entity of entities) {
            await bot.placeEntity(entity.name, entity.position, entity.direction)
        }
    }

    async saveWorldAndServer(): Promise<void> {
        await this.saveWorldInMap()
        await FactorioApi.saveServer()
    }

    async testCheatStuff(): Promise<void> {
        await FactorioApi.cheatTechnology("automation");
        await FactorioApi.cheatTechnology("logistics");
        for(const bot of this.bots) {
            await bot.cheatItem(Entities.burnerMiningDrill, 50);
            await bot.cheatItem(Entities.stoneFurnace, 50);
            await bot.cheatItem(Entities.offshorePump, 5);
            await bot.cheatItem(Entities.steamEngine, 5);
            await bot.cheatItem(Entities.boiler, 2);
            await bot.cheatItem(Entities.splitter, 50);
            await bot.cheatItem(Entities.smallElectricPole, 50);
            await bot.cheatItem(Entities.pipe, 50);
            await bot.cheatItem(Entities.pipeToGround, 50);
            await bot.cheatItem(Entities.transportBelt, 50);
            await bot.cheatItem(Entities.ironChest, 10);
            await bot.cheatItem(Entities.ironPlate, 200);
            await bot.cheatItem(Entities.ironGearWheel, 200);
            await bot.cheatItem(Entities.copperPlate, 200);
            await bot.cheatItem(Entities.coal, 200);
            await bot.cheatItem(Entities.stone, 200);
            await bot.cheatItem(Entities.electricMiningDrill, 30);
            await bot.cheatItem(Entities.transportBelt, 200);
        }
    }

    async saveWorldInMap(): Promise<void> {
        await FactorioApi.storeMapData(STORAGE_KEY, this.$store.state.world);
    }
}

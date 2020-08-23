import Vue from "vue";
import Vuex from "vuex";
import {findTask, Task} from "@/factorio-bot/task";
import {
    FactorioEntity,
    FactorioEntityPrototypeByName,
    FactorioForce,
    FactorioItemPrototypeByName,
    FactorioPlayer,
    FactorioPlayerById,
    FactorioRecipeByName, Position, StarterCoalLoop, StarterMinerChest,
    StarterMinerFurnace,
    World
} from "@/factorio-bot/types";
import {emptyWorld, positionEqual} from "@/factorio-bot/util";

Vue.use(Vuex);

export type State = {
    players: FactorioPlayerById,
    recipes: FactorioRecipeByName,
    force: FactorioForce,
    itemPrototypes: FactorioItemPrototypeByName,
    entityPrototypes: FactorioEntityPrototypeByName,
    tasks: Task[],
    world: World,
    selectedTask: Task | null
}

export const initialState: State = {
    players: {},
    recipes: {},
    itemPrototypes: {},
    entityPrototypes: {},
    force: {} as any,
    tasks: [],
    world: emptyWorld,
    selectedTask: null
}

export default new Vuex.Store({
    state: initialState,
    getters: {
        getPlayer: (state: State) => (playerId: number) => {
            return state.players[playerId.toString()]
        },
        getMainInventory: (state: State) => (playerId: number) => {
            return state.players[playerId].mainInventory
        },
    },
    mutations: {
        changeSelectedTask(state: State, task: Task | null) {
            state.selectedTask = task
        },
        updateForce(state: State, force: FactorioForce) {
            state.force = force
        },
        updateEntityPrototypes(state: State, entityPrototypes: FactorioEntityPrototypeByName) {
            state.entityPrototypes = entityPrototypes
        },
        updateItemPrototypes(state: State, itemPrototypes: FactorioItemPrototypeByName) {
            state.itemPrototypes = itemPrototypes
        },
        addStarterMinerFurnace(state: State, minerFurnace: StarterMinerFurnace) {
            state.world = {...state.world, starterMinerFurnaces: [...(state.world.starterMinerFurnaces || []), minerFurnace]}
        },
        addStarterCoalLoop(state: State, coalLoop: StarterCoalLoop) {
            state.world = {...state.world, starterCoalLoops: [...(state.world.starterCoalLoops || []), coalLoop]}
        },
        addStarterMinerChest(state: State, minerChest: StarterMinerChest) {
            state.world = {...state.world, starterMinerChests: [...(state.world.starterMinerChests || []), minerChest]}
        },
        setStarterOffshorePump(state: State, pos: Position) {
            state.world = {...state.world, starterOffshorePump: pos}
        },
        addStarterSteamEngineEntities(state: State, entities: FactorioEntity[]) {
            state.world = {...state.world, starterSteamEngineBlueprints: [...(state.world.starterSteamEngineBlueprints || []), entities] }
        },
        addStarterScienceEntities(state: State, entities: FactorioEntity[]) {
            state.world = {...state.world, starterScienceBlueprints: [...(state.world.starterScienceBlueprints || []), entities] }
        },
        updateScienceGhost(state: State, newEntity: FactorioEntity) {
            const blueprints = [...(state.world.starterScienceBlueprints || [])]
            let found = false
            for (const blueprint of blueprints) {
                for(let i=0; i<blueprint.length; i++) {
                    const entity = blueprint[i]
                    if (positionEqual(entity.position, newEntity.position)) {
                        blueprint[i] = newEntity
                        found = true
                        break
                    }
                }
                if (found) {
                    break
                }
            }
            if (!found) {
                throw new Error("no entity found to update entity ghost")
            }
            state.world = {...state.world, starterScienceBlueprints: blueprints }
        },
        updateRecipes(state: State, recipes: FactorioRecipeByName) {
            state.recipes = recipes
        },
        updateWorld(state: State, world: World) {
            state.world = world
        },
        pushTask(state: State, task: Task) {
            state.tasks = [...state.tasks, task]
        },
        addSubTask(state: State, params: { id: number, task: Task }) {
            state.tasks = [...state.tasks]
            const task = findTask(state.tasks, params.id)
            if (task) {
                task.children.push(params.task)
            }
        },
        updateTask(state: State, task: Task) {
            state.tasks = [...state.tasks]
            Object.assign(findTask(state.tasks, task.id), task)
        },
        updatePlayer(state: State, player: FactorioPlayer) {
            Vue.set(state.players, player.playerId.toString(), player)
        },
        updatePlayers(state: State, players: FactorioPlayer[]) {
            const _players: any = {}
            for (const player of players) {
                _players[player.playerId.toString()] = player
            }
            Vue.set(state, 'players', _players)
        },
    },
    modules: {},
});

use rocket::{serde::json::{self ,json, Json, Value}, config::Config};
use serde::{Deserialize, Serialize};
use tracing::debug;
use std;

#[derive(Serialize, Deserialize, Debug)]
struct PluginInfo {
    pluginId: String,
    url: String,
    receivedId: Option<String>,
}


#[get("/ai-plugin.json")]
pub async fn open_api_specification() -> Json<Value> {
  let config = Config::ADDRESS;

  debug!("{}",config);

  println!("Development URL: {}", Config::release_default().address);

  
    let routes=json!({
            "/api/tweet": {
              "get": {
                "tags": [
                  "x.com",
                  "tweet",
                  "tweet Id",
                  "Reward",
                  "Post",
                  "Clone",
                  "Snapshot",
                  "processed"
                ],
                "summary": "Get tweet post data",
                "description": "This endpoint returns an image of the post to be Rewarded along side an tweet description",
                "operationId": "get-tweet",
                "parameters": [
                  {
                    "name": "tweet_id",
                    "in": "query",
                    "description": "The tweet Id of the post to be Rewarded (NB: Its a 19 digit long numeric ID)",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "example": "1769925929940537538"
                  }
                ],
                "responses": {
                  "200": {
                    "description": "Successful response",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object",
                          "properties": {
                            "imageURL": {
                              "type": "string"
                            },
                            "description": {
                              "type": "string"
                            }
                          }
                        }
                      }
                    }
                  },
                  "400": {
                    "description": "Bad request",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object",
                          "properties": {
                            "error": {
                              "type": "string"
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            },
            "/api/tweet-contract-call": {
              "get": {
                "summary": "Reward Transaction Request data/ create transaction",
                "description": "An array of transaction objects to be signed by user to Reward Request/Reward reserve",
                "operationId": "transaction",
                "tags": [
                    "tweet",
                    "tweet Id",
                    "Reward",
                    "Craft",
                    "Reward",
                    "Derive",
                    "Produce",
                    "Duplicate",
                    "transaction"
                ],
                "parameters": [
                  {
                    "in": "query",
                    "name": "image_url",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "description": "Image URL to be Rewarded, this could be the IPFS or arweave URL in from ipfs://{CID} or ar://{Image_ID}"
                  },
                  {
                    "in": "query",
                    "name": "tweet_id",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "description": "The tweet Id of the post to be Rewarded (NB: Its a 19 digit long ID)"
                  }
                ],
                "responses": {
                  "200": {
                    "description": "Reward transactions generated successfully.",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "array",
                          "items": {
                            "type": "object",
                            "properties": {
                              "receiverId": {
                                "type": "string",
                                "description": "The account ID of the contract that will receive the transaction. CONTRACT_ID"
                              },
                              "functionCalls": {
                                "type": "array",
                                "items": {
                                  "type": "object",
                                  "properties": {
                                    "methodName": {
                                      "type": "string",
                                      "description": "The name of the method to be called on the contract."
                                    },
                                    "args": {
                                      "type": "object",
                                      "description": "Arguments for the function call.",
                                      "properties": {
                                        "tweet_id": {
                                          "type": "string"
                                        },
                                        "image_url": {
                                          "type": "string"
                                        }
                                      },
                                      "additionalProperties": true
                                    },
                                    "gas": {
                                      "type": "string",
                                      "description": "The amount of gas to attach to the transaction, in yoctoNEAR."
                                    },
                                    "amount": {
                                      "type": "string",
                                      "description": "REQUIRED: The amount of NEAR tokens to attach to the transaction, in yoctoNEAR."
                                    }
                                  },
                                  "required": [
                                    "methodName",
                                    "args",
                                    "gas",
                                    "amount"
                                  ]
                                }
                              }
                            },
                            "required": [
                              "receiverId",
                              "functionCalls"
                            ]
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
    );
    Json(json!({
            "openapi": "3.0.0",
            "info": {
              "title": "Tweet post rewarder",
              "description": "API for retrieving a digital representation of a post image along with it's description for further image generation if necessary and clones a tweet",
              "version": "1.0.0"
            },
            "servers": [
              {
                "url": if format!("{}",Config::release_default().address).eq("127.0.0.1") {
                  let bittle_config=std::fs::read_to_string("./bitte.dev.json").unwrap();
                  let plugin_info:PluginInfo  = json::serde_json::from_str(bittle_config.as_str()).unwrap();
                  plugin_info.url
              } else {
                format!("https://{}",Config::release_default().address)
              }
              }
            ],
            "x-mb": {
              "account-id": "xlassix.near",
              "assistant": {
                "name": "Post Cloner",
                "description": "An assistant that provides a digital representation of a Post as an Image with its description and generates a custom transaction for the user",
                "instructions": "Retrieve the tweet image URL and description. If images is empty, generate an image from the description. If retrieval fails, prompt the user to reserve the Reward request with a generated image",
                "tools": [
                  {
                    "type": "generate-image"
                  },
                  {
                    "type": "generate-transaction"
                  }
                ]
              }
            },
            "paths": routes
          }
    ))
}